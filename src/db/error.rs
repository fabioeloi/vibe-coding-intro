// Database Error Handling
// Defines error types for database operations

use std::fmt;
use std::error::Error;
use std::io;

/// Represents errors that can occur during database operations
#[derive(Debug)]
pub enum DatabaseError {
    /// Error connecting to the database
    Connection(String),
    /// Error executing a query
    Query(String),
    /// Error with a transaction
    Transaction(String),
    /// Error with data serialization/deserialization
    Data(String),
    /// Schema error (e.g., missing table)
    Schema(String),
    /// Migration error
    Migration(String),
    /// Lock error (mutex)
    Lock(String),
    /// I/O error
    Io(io::Error),
    /// Other database error
    Other(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DatabaseError::Connection(msg) => write!(f, "Database connection error: {}", msg),
            DatabaseError::Query(msg) => write!(f, "Query error: {}", msg),
            DatabaseError::Transaction(msg) => write!(f, "Transaction error: {}", msg),
            DatabaseError::Data(msg) => write!(f, "Data error: {}", msg),
            DatabaseError::Schema(msg) => write!(f, "Schema error: {}", msg),
            DatabaseError::Migration(msg) => write!(f, "Migration error: {}", msg),
            DatabaseError::Lock(msg) => write!(f, "Lock error: {}", msg),
            DatabaseError::Io(err) => write!(f, "I/O error: {}", err),
            DatabaseError::Other(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl Error for DatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DatabaseError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for DatabaseError {
    fn from(err: io::Error) -> Self {
        DatabaseError::Io(err)
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(err: rusqlite::Error) -> Self {
        DatabaseError::Query(err.to_string())
    }
}

/// Result type for database operations
pub type Result<T> = std::result::Result<T, DatabaseError>;
