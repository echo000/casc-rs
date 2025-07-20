/// Represents all possible errors that can occur in the CASC library.
///
/// This enum is used throughout the crate to provide detailed error information for
/// operations that may fail, such as file access, data validation, and I/O operations.
#[derive(Debug)]
pub enum CascError {
    /// Represents an error that occurs when a file is not found in the CASC storage.
    FileNotFound(String),
    /// Represents an error that occurs when a file is corrupted or invalid.
    FileCorrupted(String),
    /// Represents an error that occurs when the data in a file is invalid.
    InvalidData(String),
    /// Represents an error that occurs when a file is not supported by the CASC storage.
    UnsupportedFileType(String),
    /// Represents an error that occurs during I/O operations.
    Io(std::io::Error),
    /// Represents an error that occurs for any other reason not covered by the above variants.
    Other(String),
}

/// Provides a user-friendly string representation for each error variant in `CascError`.
impl std::fmt::Display for CascError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CascError::InvalidData(err) => write!(f, "Invalid data: {err}"),
            CascError::FileNotFound(name) => write!(f, "File not found: {name}"),
            CascError::FileCorrupted(name) => write!(f, "File is corrupted: {name}"),
            CascError::UnsupportedFileType(name) => write!(f, "Unsupported file type: {name}"),
            CascError::Io(err) => write!(f, "I/O error: {err}"),
            CascError::Other(err) => write!(f, "CASC error: {err}"),
        }
    }
}

/// Implements the standard error trait for `CascError`, allowing it to be used with
/// error chaining and other error handling utilities.
impl std::error::Error for CascError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CascError::Io(err) => Some(err),
            _ => None,
        }
    }
}

/// Allows automatic conversion from `std::io::Error` to `CascError`.
impl From<std::io::Error> for CascError {
    fn from(error: std::io::Error) -> Self {
        CascError::Io(error)
    }
}
