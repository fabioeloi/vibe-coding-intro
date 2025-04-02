// Database Module for Safari History Knowledge Graph
// Handles SQLite database operations for storing and retrieving history data

// Module organization:
// - connection.rs: Database connection management
// - models.rs: ORM-like data models
// - operations.rs: CRUD operations
// - migrations.rs: Schema migrations and initialization
// - error.rs: Error handling

pub mod connection;
pub mod models;
pub mod operations;
pub mod migrations;
pub mod error;

pub use connection::DatabaseConnection;
pub use models::{VisitRecord, UrlRecord, MetadataRecord};
pub use operations::{insert_history_data, search_history, get_stats};
pub use error::{DatabaseError, Result};

/// Initialize the database, creating schema if needed
pub fn initialize_database(db_path: &std::path::Path) -> Result<DatabaseConnection> {
    // Create connection
    let conn = connection::DatabaseConnection::new(db_path)?;
    
    // Apply migrations to ensure schema is up-to-date
    migrations::apply_migrations(&conn)?;
    
    Ok(conn)
}
