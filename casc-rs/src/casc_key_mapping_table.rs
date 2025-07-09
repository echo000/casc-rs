use base64::prelude::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Debug)]
pub struct CascKeyMappingTable {
    version: u16,
    bucket_index: u8,
    extra_byte: u8,
    encoded_size_length: u8,
    storage_offset_length: u8,
    encoding_key_length: u8,
    file_offset_bits: u8,
    file_offset_mask: u64,
    file_size: u64,
}

#[derive(Debug)]
pub struct CascKeyMappingTableEntry {
    pub encoding_key: Vec<u8>,
    pub offset: u64,
    pub size: u32,
    pub archive_index: u32,
}

impl CascKeyMappingTable {
    pub fn new(
        file_name: &PathBuf,
        entries: &mut HashMap<String, CascKeyMappingTableEntry>,
    ) -> io::Result<Self> {
        let mut file = File::open(file_name)?;

        let header_size = file.read_u32::<LittleEndian>()?;
        let header_hash = file.read_u32::<LittleEndian>()?;

        let version = file.read_u16::<LittleEndian>()?;
        let bucket_index = file.read_u8()?;
        let extra_byte = file.read_u8()?;
        let encoded_size_length = file.read_u8()?;
        let storage_offset_length = file.read_u8()?;
        let encoding_key_length = file.read_u8()?;
        let file_offset_bits = file.read_u8()?;
        let file_offset_mask = (1 << file_offset_bits) - 1;
        let file_size = file.read_u64::<LittleEndian>()?;

        if encoded_size_length != 4 && encoding_key_length != 9 && storage_offset_length != 5 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Data Sizes in Key Mapping Table",
            ));
        }

        // Align to next 0x10 boundary after adding 0x17
        let pos = file.stream_position()?;
        let new_pos = (pos + 0x17) & 0xFFFFFFF0;
        file.seek(SeekFrom::Start(new_pos))?;

        let table_size = file.read_u32::<LittleEndian>()?;
        let table_hash = file.read_u32::<LittleEndian>()?;

        let entry_size =
            (encoded_size_length + storage_offset_length + encoding_key_length) as usize;
        let table = CascKeyMappingTable {
            version,
            bucket_index,
            extra_byte,
            encoded_size_length,
            storage_offset_length,
            encoding_key_length,
            file_offset_bits,
            file_offset_mask: file_offset_mask as u64,
            file_size,
        };

        let mut entry_buffer = vec![0u8; entry_size];
        for _ in (0..table_size).step_by(entry_size) {
            file.read_exact(&mut entry_buffer)?;
            let entry = CascKeyMappingTableEntry::new(&entry_buffer, &table);
            let encoded = BASE64_STANDARD.encode(&entry.encoding_key);
            entries.insert(encoded, entry);
        }

        Ok(table)
    }
}

impl CascKeyMappingTableEntry {
    fn new(buffer: &[u8], table: &CascKeyMappingTable) -> Self {
        let mut packed_offset_and_index: u64 = 0;

        for i in 0..5 {
            packed_offset_and_index = (packed_offset_and_index << 8)
                | buffer[(i + table.encoding_key_length) as usize] as u64;
        }
        let mut size: u32 = 0;
        for i in (0..4).rev() {
            size = (size << 8)
                | buffer[(i + table.encoding_key_length + table.storage_offset_length) as usize]
                    as u32;
        }
        let archive_index = (packed_offset_and_index >> table.file_offset_bits) as u32;
        let offset = packed_offset_and_index & table.file_offset_mask;

        let encoding_key = buffer[..table.encoding_key_length as usize].to_vec();

        CascKeyMappingTableEntry {
            encoding_key,
            offset,
            size,
            archive_index,
        }
    }
}
