use crate::entry::Entry;
use crate::error::CascError;
use crate::ext::io_ext::{ArrayReadExt, ReadExt, SeekExt};
use crate::path_table_node_flags::PathTableNodeFlags;
use crate::span_info::SpanInfo;
use byteorder::{BigEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::string::String;

/// Represents the header of a TVFS root structure in a CASC archive.
///
/// This header contains metadata about the TVFS tables and their locations.
#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct TVFSHeader {
    pub signature: u32,
    pub format_version: u8,
    pub header_size: u8,
    pub encoding_key_size: u8,
    pub patch_key_size: u8,
    pub flags: i32,
    pub path_table_offset: i32,
    pub path_table_size: i32,
    pub vfs_table_offset: i32,
    pub vfs_table_size: i32,
    pub cft_table_offset: i32,
    pub cft_table_size: i32,
    pub max_depth: u16,
}

impl TVFSHeader {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            signature: reader.read_u32::<BigEndian>()?,
            format_version: reader.read_u8()?,
            header_size: reader.read_u8()?,
            encoding_key_size: reader.read_u8()?,
            patch_key_size: reader.read_u8()?,
            flags: reader.read_i32::<BigEndian>()?,
            path_table_offset: reader.read_i32::<BigEndian>()?,
            path_table_size: reader.read_i32::<BigEndian>()?,
            vfs_table_offset: reader.read_i32::<BigEndian>()?,
            vfs_table_size: reader.read_i32::<BigEndian>()?,
            cft_table_offset: reader.read_i32::<BigEndian>()?,
            cft_table_size: reader.read_i32::<BigEndian>()?,
            max_depth: reader.read_u16::<BigEndian>()?,
        })
    }
}

/// Represents a node in the TVFS path table.
///
/// Each node may represent a directory or file path component.
#[derive(Debug, Default, Clone)]
pub struct PathTableNode {
    pub name: String,
    pub flags: PathTableNodeFlags,
    pub value: Option<i32>,
}

/// Handles the TVFS root structure, including path and VFS tables, for a CASC archive.
///
/// Provides access to file entries and table readers for further processing.
#[derive(Debug)]
pub struct TVFSRootHandler {
    pub path_table_reader: Cursor<Vec<u8>>,
    pub vfs_table_reader: Cursor<Vec<u8>>,
    pub cft_table_reader: Cursor<Vec<u8>>,
    pub header: TVFSHeader,
    pub file_entries: HashMap<String, Entry>,
}

impl TVFSRootHandler {
    pub fn new<R: Read + Seek>(stream: &mut R) -> Result<Self, CascError> {
        stream.seek(SeekFrom::Start(0))?;
        let mut reader = BufReader::new(stream);
        let header = TVFSHeader::read(&mut reader)?;

        // Read tables into memory
        reader.seek(SeekFrom::Start(header.path_table_offset as u64))?;
        let path_table_buf = reader.read_array::<u8>(header.path_table_size as usize)?;

        reader.seek(SeekFrom::Start(header.vfs_table_offset as u64))?;
        let vfs_table_buf = reader.read_array::<u8>(header.vfs_table_size as usize)?;

        reader.seek(SeekFrom::Start(header.cft_table_offset as u64))?;
        let cft_table_buf = reader.read_array::<u8>(header.cft_table_size as usize)?;

        let mut handler = TVFSRootHandler {
            path_table_reader: Cursor::new(path_table_buf),
            vfs_table_reader: Cursor::new(vfs_table_buf),
            cft_table_reader: Cursor::new(cft_table_buf),
            header,
            file_entries: HashMap::new(),
        };

        let end =
            handler.path_table_reader.position() + handler.path_table_reader.get_ref().len() as u64;
        handler.parse(end, String::with_capacity(255))?;

        Ok(handler)
    }

    fn parse_path_node(&mut self) -> Result<PathTableNode, CascError> {
        let mut entry = PathTableNode::default();

        let mut buf = self.path_table_reader.peek_byte()?;

        if buf == 0 {
            entry.flags |= PathTableNodeFlags::PATH_SEPARATOR_PRE;
            self.path_table_reader.skip(1)?;
            buf = self.path_table_reader.peek_byte()?;
        }

        if buf < 0x7F && buf != 0xFF {
            self.path_table_reader.skip(1)?;
            let chars = self.path_table_reader.read_chars(buf as usize)?;
            entry.name = chars.into_iter().collect();
            buf = self.path_table_reader.peek_byte()?;
        }

        if buf == 0 {
            entry.flags |= PathTableNodeFlags::PATH_SEPARATOR_POST;
            self.path_table_reader.skip(1)?;
            buf = self.path_table_reader.peek_byte()?;
        }

        if buf == 0xFF {
            self.path_table_reader.skip(1)?;
            entry.value = Some(self.path_table_reader.read_i32::<BigEndian>()?);
            entry.flags |= PathTableNodeFlags::IS_NODE_VALUE;
        } else {
            entry.flags |= PathTableNodeFlags::PATH_SEPARATOR_POST;
        }

        Ok(entry)
    }

    fn add_entry(&mut self, name: String, vfs_info_pos: u64) -> Result<(), CascError> {
        self.vfs_table_reader.set_position(vfs_info_pos);

        let span_count = self.vfs_table_reader.read_u8()?;
        let mut spans = Vec::new();
        for _ in 0..span_count {
            let _ref_file_offset = self.vfs_table_reader.read_i32::<BigEndian>()?;
            let _size_of_span = self.vfs_table_reader.read_i32::<BigEndian>()?;
            let cft_offset = Self::read_variable_size_int(
                &mut self.vfs_table_reader,
                self.header.cft_table_size as usize,
            )?;

            self.cft_table_reader.set_position(cft_offset as u64);

            let mut buf = vec![0u8; self.header.encoding_key_size as usize];
            self.cft_table_reader.read_exact(&mut buf)?;
            spans.push(SpanInfo::new_with_encoding_key(buf));
        }
        let mut entry = Entry::new_with_spans(name, spans);

        self.file_entries.insert(entry.name.clone(), entry);
        Ok(())
    }

    fn read_variable_size_int<R: Read + Seek>(
        reader: &mut R,
        data_size: usize,
    ) -> Result<u32, CascError> {
        let data = if data_size > 0xFFFFFF {
            // Read 4 bytes (32 bits, big-endian)
            reader.read_u32::<BigEndian>()
        } else if data_size > 0xFFFF {
            // Read 3 bytes (24 bits, big-endian)
            let mut buf = [0u8; 3];
            reader.read_exact(&mut buf)?;
            Ok(((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32))
        } else if data_size > 0xFF {
            // Read 2 bytes (16 bits, big-endian)
            reader.read_u16::<BigEndian>().map(|v| v as u32)
        } else {
            // Read 1 byte
            reader.read_u8().map(|v| v as u32)
        };
        data.map_err(|e| CascError::Io(e))
    }

    fn parse(&mut self, end: u64, mut builder: String) -> Result<(), CascError> {
        let current_size = builder.len();

        while self.path_table_reader.position() < end {
            let entry = self.parse_path_node()?;

            // Build name with flags
            if entry.flags.has_flag(PathTableNodeFlags::PATH_SEPARATOR_PRE) {
                builder.push('\\');
            }
            builder.push_str(&entry.name);
            if entry
                .flags
                .has_flag(PathTableNodeFlags::PATH_SEPARATOR_POST)
            {
                builder.push('\\');
            }

            if entry.flags.has_flag(PathTableNodeFlags::IS_NODE_VALUE) {
                if let Some(val) = entry.value {
                    if (val as u32 & 0x8000_0000) != 0 {
                        let folder_size = val & 0x7FFF_FFFF;
                        let folder_start = self.path_table_reader.position();
                        let folder_end = folder_start + folder_size as u64 - 4;
                        self.parse(folder_end, builder.clone())?;
                    } else {
                        self.add_entry(builder.clone(), entry.value.unwrap() as u64)?;
                    }
                }
                // Reset builder to original
                builder.truncate(current_size);
            }
        }
        Ok(())
    }
}
