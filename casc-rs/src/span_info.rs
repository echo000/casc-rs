use base64::prelude::*;

#[derive(Debug)]
pub struct SpanInfo {
    pub content_key: Option<Vec<u8>>,
    pub encoding_key: Vec<u8>,
    pub size: Option<usize>,
    pub base64_content_key: Option<String>,
    pub base64_encoding_key: String,
}

impl SpanInfo {
    pub fn new_with_encoding_key(e_key: Vec<u8>) -> Self {
        let base64_encoding_key = BASE64_STANDARD.encode(&e_key);
        Self {
            content_key: None,
            encoding_key: e_key,
            size: None,
            base64_content_key: None,
            base64_encoding_key,
        }
    }

    pub fn new_with_content_key(c_key: Vec<u8>, e_key: Vec<u8>, size: usize) -> Self {
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
