#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct CascSpanHeader {
    pub encoding_key: [u8; 16],
    pub content_size: i32,
    pub flags: u16,
    pub jenkins_hash: u32,
    pub checksum: u32,
}

impl CascSpanHeader {
    pub fn new() -> Self {
        Self {
            encoding_key: [0; 16],
            content_size: 0,
            flags: 0,
            jenkins_hash: 0,
            checksum: 0,
        }
    }
}
