use crate::span_info::SpanInfo;

/// Entry module for CASC file entries.
///
/// This module defines the `Entry` struct, which represents a file entry in the CASC storage,
/// including its name and associated spans.

/// Represents a file entry in the CASC storage.
///
/// Each `Entry` contains the file's name and a list of spans describing the file's data segments.
#[derive(Debug)]
pub struct Entry {
    /// The name of the file entry.
    pub name: String,
    /// The spans associated with this entry, describing segments of the file's data.
    pub(crate) spans: Vec<SpanInfo>,
}

impl Entry {
    pub(crate) fn new_with_spans(name: String, spans: Vec<SpanInfo>) -> Self {
        Self { name, spans }
    }
}
