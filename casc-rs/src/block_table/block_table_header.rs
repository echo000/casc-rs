/// Block Table Header
/// Represents the header of a block table in a CASC storage.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct BlockTableHeader {
    /// The signature identifying the block table.
    pub(crate) signature: u32,
    /// The size of the header in bytes.
    pub(crate) header_size: u32,
    /// The format version of the table.
    pub(crate) table_format: u8,
    /// The number of frames in the table (i24).
    pub(crate) frame_count: [u8; 3],
}
