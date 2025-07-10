/// Represents information about a file in the CASC storage.
#[derive(Debug)]
pub struct CascFileInfo {
    /// The name of the file.
    file_name: String,
    /// The size of the file in bytes.
    file_size: i64,
    /// Whether the file is local to the storage.
    is_local: bool,
}

impl CascFileInfo {
    pub(crate) fn new(file_name: String, file_size: i64, is_local: bool) -> Self {
        Self {
            file_name,
            file_size,
            is_local,
        }
    }

    /// Returns the name of the file.
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Sets the name of the file.
    pub(crate) fn set_file_name(&mut self, name: String) {
        self.file_name = name;
    }

    /// Returns the size of the file in bytes.
    pub fn file_size(&self) -> i64 {
        self.file_size
    }

    /// Sets the size of the file in bytes.
    pub(crate) fn set_file_size(&mut self, size: i64) {
        self.file_size = size;
    }

    /// Returns whether the file is local to the storage.
    pub fn is_local(&self) -> bool {
        self.is_local
    }

    /// Sets whether the file is local to the storage.
    pub(crate) fn set_is_local(&mut self, is_local: bool) {
        self.is_local = is_local;
    }
}
