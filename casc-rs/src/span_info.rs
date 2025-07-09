/// This module defines the `SpanInfo` struct, which represents information about a span of data
/// within a CASC archive, including keys, size, and their base64 representations.
use base64::prelude::*;

/// Represents information about a span of data in a CASC archive.
///
/// A `SpanInfo` contains the binary and base64 representations of the content and encoding keys,
/// as well as the size of the span if known.
#[derive(Debug)]
pub(crate) struct SpanInfo {
    /// The binary content key, if present.
    pub(crate) content_key: Option<Vec<u8>>,
    /// The binary encoding key.
    pub(crate) encoding_key: Vec<u8>,
    /// The size of the span, if known.
    pub(crate) size: Option<usize>,
    /// The base64-encoded content key, if present.
    pub(crate) base64_content_key: Option<String>,
    /// The base64-encoded encoding key.
    pub(crate) base64_encoding_key: String,
}

impl SpanInfo {
    pub(crate) fn new_with_encoding_key(e_key: Vec<u8>) -> Self {
        let base64_encoding_key = BASE64_STANDARD.encode(&e_key);
        Self {
            content_key: None,
            encoding_key: e_key,
            size: None,
            base64_content_key: None,
            base64_encoding_key,
        }
    }

    pub(crate) fn new_with_content_key(c_key: Vec<u8>, e_key: Vec<u8>, size: usize) -> Self {
        let base64_content_key = BASE64_STANDARD.encode(&c_key);
        let base64_encoding_key = BASE64_STANDARD.encode(&e_key);
        Self {
            content_key: Some(c_key),
            encoding_key: e_key,
            size: Some(size),
            base64_content_key: Some(base64_content_key),
            base64_encoding_key,
        }
    }
}
