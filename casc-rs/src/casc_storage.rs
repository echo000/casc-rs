use crate::{
    block_table::{block_table_entry::BlockTableEntry, block_table_header::BlockTableHeader},
    casc_build_info::CascBuildInfo,
    casc_config::CascConfig,
    casc_file_frame::CascFileFrame,
    casc_file_info::CascFileInfo,
    casc_file_reader::CascFileReader,
    casc_file_span::CascFileSpan,
    casc_key_mapping_table::{CascKeyMappingTable, CascKeyMappingTableEntry},
    casc_span_header::CascSpanHeader,
    error::CascError,
    ext::io_ext::{ArrayReadExt, StructReadExt},
    tvfs_root_handler::TVFSRootHandler,
};
use base64::prelude::*;
use glob::glob;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct CascStorage {
    entries: HashMap<String, CascKeyMappingTableEntry>,
    key_mapping_tables: Vec<CascKeyMappingTable>,
    root_handler: TVFSRootHandler,
    build_info: CascBuildInfo,
    config: CascConfig,
    storage_path: String,
    data_path: String,
    data_files: Vec<File>,
    pub files: Vec<CascFileInfo>,
}

impl CascStorage {
    pub fn open<P: AsRef<Path>>(
        folder: P,
        handler: Option<TVFSRootHandler>,
    ) -> Result<Self, CascError> {
        let f = folder.as_ref();
        let data_path = f.join("Data").join("data");

        let data_path = data_path.display().to_string();
        let storage_path = f.display().to_string();
        let build_info = Self::load_build_info(&storage_path)?;
        let config = Self::load_config_info(&build_info, &storage_path)?;

        let idx_files = fs::read_dir(&data_path)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "idx")
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();

        let mut entries = HashMap::new();
        let mut key_mapping_tables = Vec::new();
        for idx_file in idx_files {
            let key_table = CascKeyMappingTable::new(&idx_file.path(), &mut entries)?;
            key_mapping_tables.push(key_table);
        }

        let data_files = Self::load_data_files(&data_path)?;
        let root_handler = Self::load_root_handler(&config, handler, &data_files, &entries)?;

        let files = Self::load_files(&root_handler, &entries)?;

        Ok(CascStorage {
            entries,
            key_mapping_tables,
            root_handler,
            build_info,
            config,
            storage_path,
            data_path,
            data_files,
            files,
        })
    }

    fn load_build_info(storage_path: &str) -> Result<CascBuildInfo, CascError> {
        fn find_build_info<P: AsRef<Path>>(dir: P) -> Option<PathBuf> {
            for entry in fs::read_dir(dir).ok()? {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.file_name() == Some(".build.info".as_ref()) {
                    return Some(path);
                } else if path.is_dir() {
                    if let Some(found) = find_build_info(&path) {
                        return Some(found);
                    }
                }
            }
            None
        }

        if let Some(path) = find_build_info(storage_path) {
            let mut build_info = CascBuildInfo::new();
            build_info.load(&path)?;
            Ok(build_info)
        } else {
            Err(CascError::FileNotFound(
                "Failed to locate Build Info".into(),
            ))
        }
    }

    fn load_config_info(
        build_info: &CascBuildInfo,
        storage_path: &str,
    ) -> Result<CascConfig, CascError> {
        fn find_config<P: AsRef<Path>>(dir: P, build_key: &str) -> Option<std::path::PathBuf> {
            for entry in fs::read_dir(dir).ok()? {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.file_name() == Some(build_key.as_ref()) {
                    return Some(path);
                } else if path.is_dir() {
                    if let Some(found) = find_config(&path, build_key) {
                        return Some(found);
                    }
                }
            }
            None
        }

        let mut config = CascConfig::new();
        let build_key = build_info.get("Build Key", "");
        if let Some(path) = find_config(storage_path, &build_key) {
            config.load(&path)?;
            Ok(config)
        } else {
            Err(CascError::FileNotFound(
                "Failed to locate Config Info".into(),
            ))
        }
    }
    fn load_data_files(data_path: &str) -> Result<Vec<File>, CascError> {
        let pattern = format!("{}/data.*", data_path);
        let mut indexed_files: Vec<(usize, PathBuf)> = Vec::new();

        for entry in glob(&pattern).expect("Failed to read glob pattern") {
            let path = entry.map_err(|e| CascError::Other(format!("{e}")))?;
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if let Ok(index) = ext.parse::<usize>() {
                    indexed_files.push((index, path));
                }
            }
        }

        let max_index = indexed_files.iter().map(|(i, _)| *i).max().unwrap_or(0);
        let mut data_files: Vec<Option<File>> = (0..=max_index).map(|_| None).collect();

        for (index, path) in indexed_files {
            let file = File::open(path)?;
            data_files[index] = Some(file);
        }

        data_files
            .into_iter()
            .map(|opt| opt.ok_or_else(|| CascError::FileNotFound("Missing data file".to_string())))
            .collect()
    }

    fn load_root_handler(
        config: &CascConfig,
        handler: Option<TVFSRootHandler>,
        data_files: &[File],
        entries: &HashMap<String, CascKeyMappingTableEntry>,
    ) -> Result<TVFSRootHandler, CascError> {
        // If handler already exists, just return it
        if let Some(existing_handler) = handler {
            return Ok(existing_handler);
        }

        // Get the "vfs-root" key from config
        let key = config
            .get("vfs-root")
            .ok_or_else(|| CascError::Other("vfs-root not in config".to_string()))?;

        let hex_bytes = hex::decode(&key.values[1]) // assuming config.get returns a struct with .values[1]
            .map_err(|_| {
                CascError::InvalidData("Invalid hex in vfs-root".to_string())
            })?;

        let base64 = BASE64_STANDARD.encode(&hex_bytes);
        let base64_key = &base64[0..12];

        // Look up entry by transformed key
        let entry = entries.get(base64_key).ok_or_else(|| {
            CascError::FileNotFound(format!("Entry not found in entries: {base64_key}"))
        })?;

        // Open the stream
        let mut stream = Self::open_file_from_entry(entry, data_files)
            .map_err(|_| CascError::Other("Failed to open entry file".to_string()))?;

        // Read the first 4 bytes
        let mut header_buf = [0u8; 4];
        stream.read_exact(&mut header_buf)?;

        // Reset stream position
        stream.seek(SeekFrom::Start(0))?;

        // Match on header
        let root_handler = match u32::from_le_bytes(header_buf) {
            0x53465654 => TVFSRootHandler::new(&mut stream)?,
            _ => return Err(CascError::InvalidData("Invalid VFS header".to_string())),
        };

        Ok(root_handler)
    }

    fn load_files(
        handler: &TVFSRootHandler,
        entries: &HashMap<String, CascKeyMappingTableEntry>,
    ) -> Result<Vec<CascFileInfo>, CascError> {
        let mut files = Vec::new();
        for (name, entry) in &handler.file_entries {
            let mut info = CascFileInfo {
                file_name: name.clone(),
                is_local: true,
                file_size: 0,
            };
            for span_info in &entry.spans {
                match entries.get(&span_info.base64_encoding_key) {
                    Some(entry1) => info.file_size += entry1.size as i64,
                    None => {
                        info.is_local = false;
                        info.file_size = 0;
                        break;
                    }
                }
            }
            files.push(info);
        }
        Ok(files)
    }

    pub fn open_file(&self, entry: &str) -> Result<CascFileReader, CascError> {
        let entry = self
            .root_handler
            .file_entries
            .get(entry)
            .ok_or_else(|| CascError::FileNotFound(format!("Entry not found: {entry}")))?;
        let mut virtual_offset = 0u64;
        let mut spans: Vec<CascFileSpan<File>> = Vec::new();

        for span in &entry.spans {
            if let Some(e) = self.entries.get(&span.base64_encoding_key) {
                let mut reader = self.data_files[e.archive_index as usize].try_clone()?;
                reader.seek(SeekFrom::Start(e.offset))?;

                // Read and discard the span header
                let _ = reader.read_struct::<CascSpanHeader>()?;
                let header = reader.read_struct::<BlockTableHeader>()?;

                if header.signature != 0x45544C42 {
                    return Err(CascError::InvalidData(
                        "Invalid Block Table Header signature".to_string(),
                    )
                    .into());
                }

                // Bitshift the i24BE to u32 LE
                let frame_count = header.frame_count[2] as u32
                    | (header.frame_count[1] as u32) << 8
                    | (header.frame_count[0] as u32) << 16;
                let block_table_frames =
                    reader.read_array::<BlockTableEntry>(frame_count as usize)?;
                let mut archive_offset = reader.stream_position()?;

                let mut span_archive_offset = archive_offset;
                let mut span_virtual_start_offset = virtual_offset;
                let mut span_virtual_end_offset = virtual_offset;
                let mut frames = Vec::new();

                for block_table_frame in block_table_frames {
                    //Swap from BE to LE
                    let encoded_size = i32::from_be(block_table_frame.encoded_size) as u32;
                    let content_size = i32::from_be(block_table_frame.content_size) as u32;
                    let frame = CascFileFrame {
                        archive_offset,
                        encoded_size,
                        content_size,
                        virtual_start_offset: virtual_offset,
                        virtual_end_offset: virtual_offset + content_size as u64,
                    };
                    span_virtual_end_offset += frame.content_size as u64;
                    archive_offset += encoded_size as u64;
                    virtual_offset += content_size as u64;
                    frames.push(frame);
                }

                let mut new_span = CascFileSpan::<File>::new(
                    reader,
                    span_virtual_start_offset,
                    virtual_offset,
                    span_archive_offset,
                    frames,
                );
                spans.push(new_span);
            };
        }
        Ok(CascFileReader::new(spans, virtual_offset))
    }

    pub(crate) fn open_file_from_entry(
        entry: &CascKeyMappingTableEntry,
        data_files: &[File],
    ) -> Result<CascFileReader, CascError> {
        let mut virtual_offset = 0u64;
        let mut spans: Vec<CascFileSpan<File>> = Vec::new();

        // Clone the file handle for independent reading
        let mut reader = data_files[entry.archive_index as usize].try_clone()?;
        reader.seek(SeekFrom::Start(entry.offset))?;

        // Read and discard the span header
        let _ = reader.read_struct::<CascSpanHeader>()?;
        let header = reader.read_struct::<BlockTableHeader>()?;

        if header.signature != 0x45544C42 {
            return Err(CascError::InvalidData(
                "Invalid Block Table Header signature".to_string(),
            ));
        }

        // Bitshift the i24BE to u32 LE
        let frame_count = header.frame_count[2] as u32
            | (header.frame_count[1] as u32) << 8
            | (header.frame_count[0] as u32) << 16;
        let block_table_frames = reader.read_array::<BlockTableEntry>(frame_count as usize)?;
        let mut archive_offset = reader.stream_position()?;

        let mut span_archive_offset = archive_offset;
        let mut span_virtual_start_offset = virtual_offset;
        let mut span_virtual_end_offset = virtual_offset;
        let mut frames = Vec::new();

        for block_table_frame in block_table_frames {
            //Swap from BE to LE
            let encoded_size = i32::from_be(block_table_frame.encoded_size) as u32;
            let content_size = i32::from_be(block_table_frame.content_size) as u32;
            let frame = CascFileFrame {
                archive_offset,
                encoded_size,
                content_size,
                virtual_start_offset: virtual_offset,
                virtual_end_offset: virtual_offset + content_size as u64,
            };
            span_virtual_end_offset += frame.content_size as u64;
            archive_offset += encoded_size as u64;
            virtual_offset += content_size as u64;
            frames.push(frame);
        }

        let mut new_span = CascFileSpan::<File>::new(
            reader,
            span_virtual_start_offset,
            virtual_offset,
            span_archive_offset,
            frames,
        );
        spans.push(new_span);

        Ok(CascFileReader::new(spans, virtual_offset))
    }
}
