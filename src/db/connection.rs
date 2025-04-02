// Database Connection Manager
// Handles SQLite connection creation and management

use std::path::{Path, PathBuf};
use rusqlite::{Connection, OpenFlags};
use std::sync::{Arc, Mutex};

use super::error::{DatabaseError, Result};

/// Represents a connection to the database
pub struct DatabaseConnection {
    /// Path to the database file
    pub path: PathBuf,
    /// The SQLite connection wrapped in Arc<Mutex<>> for thread safety
    connection: Arc<Mutex<Connection>>,
}

impl DatabaseConnection {
    /// Creates a new database connection
    pub fn new(path: &Path) -> Result<Self> {
        // Open the SQLite database with appropriate flags
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        ).map_err(|e| DatabaseError::Connection(e.to_string()))?;
        
        // Enable foreign keys support
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        
        // Set some sensible defaults for performance
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = 1000;
            PRAGMA temp_store = MEMORY;
        ").map_err(|e| DatabaseError::Query(e.to_string()))?;
        
        Ok(Self {
            path: path.to_path_buf(),
            connection: Arc::new(Mutex::new(conn)),
        })
    }
    
    /// Gets a reference to the connection for operations
    /// This ensures thread safety with the mutex lock
    pub fn get(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection.lock()
            .map_err(|_| DatabaseError::Lock("Failed to acquire database lock".to_string()))
    }
    
    /// Executes a function with the database connection
    /// This pattern ensures the lock is always released
    pub fn with_connection<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self.get()?;
        f(&conn)
    }
    
    /// Begins a transaction
    pub fn transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self.get()?;
        
        let tx = conn.transaction()
            .map_err(|e| DatabaseError::Transaction(e.to_string()))?;
            
        match f(&tx) {
            Ok(result) => {
                tx.commit().map_err(|e| DatabaseError::Transaction(e.to_string()))?;
                Ok(result)
            },
            Err(e) => {
                // Transaction will be rolled back when dropped
                Err(e)
            }
        }
    }
    
    /// Executes a batch of SQL statements
    pub fn execute_batch(&self, sql: &str) -> Result<()> {
        self.with_connection(|conn| {
            conn.execute_batch(sql)
                .map_err(|e| DatabaseError::Query(e.to_string()))?;
            Ok(())
        })
    }
    
    /// Checks if the database is initialized with the expected schema
    pub fn is_initialized(&self) -> Result<bool> {
        self.with_connection(|conn| {
            // Check if our main tables exist
            let tables = ["url", "visit", "metadata"];
            
            for table in tables {
                let query = format!(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name='{}'",
                    table
                );
                
                let exists: bool = conn.query_row(&query, [], |row| row.get(0))
                    .unwrap_or(false);
                    
                if !exists {
                    return Ok(false);
                }
            }
            
            Ok(true)
        })
    }
}
