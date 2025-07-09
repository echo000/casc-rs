use crate::error::CascError;
/// Module for handling CASC key mapping tables, which map encoding keys to file offsets and sizes.
///
/// This module provides structures and functions for parsing and working with key mapping tables
/// found in CASC storages. These tables are used to locate and access file data by encoding key.
use base64::prelude::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;

/// Represents a CASC key mapping table, which maps encoding keys to file offsets and sizes.
///
/// This struct is used to parse and store the metadata for a key mapping table in a CASC storage.
/// The table allows efficient lookup of file data by encoding key.
#[derive(Debug)]
pub struct CascKeyMappingTable {
    /// The version of the key mapping table format.
    version: u16,
    /// The bucket index used for hashing.
    bucket_index: u8,
    /// An extra byte used for format-specific purposes.
    extra_byte: u8,
    /// The length in bytes of the encoded size field.
    encoded_size_length: u8,
    /// The length in bytes of the storage offset field.
    storage_offset_length: u8,
    /// The length in bytes of the encoding key.
    encoding_key_length: u8,
    /// The number of bits used for the file offset.
    file_offset_bits: u8,
    /// The mask applied to the file offset.
    file_offset_mask: u64,
    /// The total file size represented by this table.
    file_size: u64,
}

#[derive(Debug)]
/// Represents a single entry in a CASC key mapping table.
///
/// Each entry maps an encoding key to a file offset, size, and archive index.
pub struct CascKeyMappingTableEntry {
    /// The encoding key for the file data.
    pub encoding_key: Vec<u8>,
    /// The offset of the file data within the archive.
    pub offset: u64,
    /// The size of the file data.
    pub size: u32,
    /// The index of the archive containing the file data.
    pub archive_index: u32,
}

impl CascKeyMappingTable {
    pub(crate) fn new(
        file_name: &PathBuf,
        entries: &mut HashMap<String, CascKeyMappingTableEntry>,
    ) -> Result<Self, CascError> {
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
            return Err(CascError::FileCorrupted(
                "Invalid Data Sizes in Key Mapping Table".into(),
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
