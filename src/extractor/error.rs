// Safari History Extractor - Error Handling
// Define error types for the extraction process

use std::fmt;
use std::error::Error;
use std::io;
use std::path::PathBuf;

/// Represents errors that can occur during the extraction process
#[derive(Debug)]
pub enum ExtractionError {
    /// An IO error occurred (e.g., file not found, permission denied)
    Io(io::Error),
    /// The file format was invalid or corrupted
    InvalidFormat(String),
    /// The SQLite database could not be accessed or was invalid
    Database(String),
    /// A parsing error occurred while processing the database
    Parse(String),
    /// The file was valid but had an unsupported schema or version
    UnsupportedSchema(String),
    /// Another kind of error occurred
    Other(String),
}

impl fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExtractionError::Io(err) => write!(f, "IO error: {}", err),
            ExtractionError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ExtractionError::Database(msg) => write!(f, "Database error: {}", msg),
            ExtractionError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ExtractionError::UnsupportedSchema(msg) => write!(f, "Unsupported schema: {}", msg),
            ExtractionError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl Error for ExtractionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ExtractionError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for ExtractionError {
    fn from(err: io::Error) -> Self {
        ExtractionError::Io(err)
    }
}

impl From<rusqlite::Error> for ExtractionError {
    fn from(err: rusqlite::Error) -> Self {
        ExtractionError::Database(err.to_string())
    }
}

/// Result type for extraction operations
pub type Result<T> = std::result::Result<T, ExtractionError>;

/// Represents a file that could not be processed
#[derive(Debug)]
pub struct FailedFile {
    /// Path to the file that failed
    pub path: PathBuf,
    /// The error that occurred
    pub error: ExtractionError,
}

impl FailedFile {
    /// Creates a new FailedFile
    pub fn new(path: PathBuf, error: ExtractionError) -> Self {
        Self { path, error }
    }
    
    /// Returns a string description of the failure
    pub fn description(&self) -> String {
        format!("Failed to process '{}': {}", 
            self.path.display(), 
            self.error)
    }
}
