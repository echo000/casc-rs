/// This module defines the `CascSpanHeader` struct, which represents the header for a span of data
/// in a CASC archive. The header contains metadata such as encoding key, content size, flags,
/// hash, and checksum.

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
/// Represents the header for a span of data in a CASC archive.
///
/// The `CascSpanHeader` contains metadata fields used to identify and validate a span of file data.
pub(crate) struct CascSpanHeader {
    /// The encoding key for the span (typically a hash or identifier).
    pub(crate) encoding_key: [u8; 16],
    /// The size of the content in bytes.
    pub(crate) content_size: i32,
    /// Flags associated with the span.
    pub(crate) flags: u16,
    /// Jenkins hash of the span data.
    pub(crate) jenkins_hash: u32,
    /// Checksum for data integrity verification.
    pub(crate) checksum: u32,
}

impl CascSpanHeader {
    pub(crate) fn new() -> Self {
        Self {
            encoding_key: [0; 16],
            content_size: 0,
            flags: 0,
            jenkins_hash: 0,
            checksum: 0,
        }
    }
}
