/// Represents the encoding type used for entries in the block table.
///
/// This enum describes how the data in a block table entry is stored or compressed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockTableEncoderType {
    /// Plain raw data, uncompressed and unencrypted.
    Raw = 0x4E,
    /// Zlib compressed data.
    ZLib = 0x5A,
    /// Encrypted data.
    Encrypted = 0x45,
    /// Unknown or unsupported type, stores the raw byte value.
    Unknown(u8),
}

impl From<u8> for BlockTableEncoderType {
    fn from(byte: u8) -> Self {
        match byte {
            0x4E => BlockTableEncoderType::Raw,
            0x5A => BlockTableEncoderType::ZLib,
            0x45 => BlockTableEncoderType::Encrypted,
            other => BlockTableEncoderType::Unknown(other),
        }
    }
}
