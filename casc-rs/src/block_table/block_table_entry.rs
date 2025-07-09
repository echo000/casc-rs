/// Represents an entry in the CASC block table.
/// Each entry describes a block of data in the storage.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BlockTableEntry {
    /// The encoded (compressed/encrypted) size of the block.
    pub encoded_size: i32,
    /// The decoded (original) content size of the block.
    pub content_size: i32,
    /// Lower 64 bits of the block's hash.
    pub hash_lower: u64,
    /// Upper 64 bits of the block's hash.
    pub hash_upper: u64,
}
