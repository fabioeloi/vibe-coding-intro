// Database Migrations
// Handles schema creation and updates

use std::fs;
use std::path::Path;
use rusqlite::Connection;

use super::connection::DatabaseConnection;
use super::error::{DatabaseError, Result};

/// Applies all migrations to ensure the database schema is up-to-date
pub fn apply_migrations(conn: &DatabaseConnection) -> Result<()> {
    // Check if the database has been initialized
    if !conn.is_initialized()? {
        // Apply the initial schema if not
        apply_initial_schema(conn)?;
    }
    
    // Additional migrations can be applied here in the future
    // Each migration should be versioned and only applied if needed
    
    Ok(())
}

/// Applies the initial database schema
pub fn apply_initial_schema(conn: &DatabaseConnection) -> Result<()> {
    // Load the schema SQL from our schema file
    let schema_sql = include_str!("../../database/schema.sql");
    
    // Execute the schema creation
    conn.execute_batch(schema_sql)
        .map_err(|e| DatabaseError::Migration(format!("Failed to apply initial schema: {}", e)))?;
    
    Ok(())
}

/// Gets the current schema version from the database
pub fn get_schema_version(conn: &Connection) -> Result<i32> {
    // Check if the version table exists
    let version_table_exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_version')",
        [],
        |row| row.get(0),
    ).map_err(|e| DatabaseError::Query(e.to_string()))?;
    
    if !version_table_exists {
        // Create the version table if it doesn't exist
        conn.execute(
            "CREATE TABLE schema_version (version INTEGER NOT NULL)",
            [],
        ).map_err(|e| DatabaseError::Query(e.to_string()))?;
        
        // Insert the initial version
        conn.execute(
            "INSERT INTO schema_version (version) VALUES (1)",
            [],
        ).map_err(|e| DatabaseError::Query(e.to_string()))?;
        
        return Ok(1);
    }
    
    // Get the current version
    conn.query_row(
        "SELECT version FROM schema_version LIMIT 1",
        [],
        |row| row.get(0),
    ).map_err(|e| DatabaseError::Query(e.to_string()))
}

/// Updates the schema version in the database
pub fn update_schema_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(
        "UPDATE schema_version SET version = ?",
        [version],
    ).map_err(|e| DatabaseError::Query(e.to_string()))?;
    
    Ok(())
}

/// Loads a migration SQL file from the migrations directory
pub fn load_migration_sql(version: i32) -> Result<String> {
    let migration_path = Path::new("database/migrations")
        .join(format!("v{}.sql", version));
    
    fs::read_to_string(&migration_path)
        .map_err(|e| DatabaseError::Migration(
            format!("Failed to read migration file {}: {}", migration_path.display(), e)
        ))
}
