/// Represents an entry in the CASC block table.
/// Each entry describes a block of data in the storage.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct BlockTableEntry {
    /// The encoded (compressed/encrypted) size of the block.
    pub(crate) encoded_size: i32,
    /// The decoded (original) content size of the block.
    pub(crate) content_size: i32,
    /// Lower 64 bits of the block's hash.
    pub(crate) hash_lower: u64,
    /// Upper 64 bits of the block's hash.
    pub(crate) hash_upper: u64,
}
