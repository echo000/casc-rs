/// Represents information about a file in the CASC storage.
#[derive(Debug)]
pub struct CascFileInfo {
    /// The name of the file.
    pub file_name: String,
    /// The size of the file in bytes.
    pub file_size: i64,
    /// Whether the file is local to the storage.
    pub is_local: bool,
}
