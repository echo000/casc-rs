/// Represents a frame within a CASC file, describing a segment of file data.
pub struct CascFileFrame {
    /// The virtual start offset of the frame within the file.
    pub virtual_start_offset: u64,
    /// The virtual end offset of the frame within the file.
    pub virtual_end_offset: u64,
    /// The offset of the frame within the archive.
    pub archive_offset: u64,
    /// The encoded (compressed/encrypted) size of the frame.
    pub encoded_size: u32,
    /// The decoded (original) content size of the frame.
    pub content_size: u32,
}
