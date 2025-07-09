use crate::casc_file_frame::CascFileFrame;
use std::io::Read;

/// Represents a span in a CASC file, including offsets and file frames.
///
/// A `CascFileSpan` describes a contiguous region of a file within the CASC storage,
/// including its offsets and the frames it contains.
pub struct CascFileSpan<R: Read> {
    /// The reader for the span (if any).
    pub(crate) span_reader: Option<R>,
    /// The virtual start offset of the span.
    pub(crate) virtual_start_offset: u64,
    /// The virtual end offset of the span.
    pub(crate) virtual_end_offset: u64,
    /// The archive offset of the span.
    pub(crate) archive_offset: u64,
    /// The file frames within this span.
    pub(crate) frames: Vec<CascFileFrame>,
}

impl<R: Read> CascFileSpan<R> {
    /// Creates a new `CascFileSpan` with default values.
    ///
    /// # Returns
    ///
    /// A `CascFileSpan` with all offsets set to zero and no frames.
    pub(crate) fn new() -> Self {
        Self {
            span_reader: None,
            virtual_start_offset: 0,
            virtual_end_offset: 0,
            archive_offset: 0,
            frames: Vec::new(),
        }
    }
}
