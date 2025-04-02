// Safari History Extractor - Safari-specific parser
// This module handles the actual extraction logic for Safari history.db files

use std::path::{Path, PathBuf};
use rusqlite::{Connection, Row, Result as SqliteResult};
use chrono::{DateTime, Utc, TimeZone};
use uuid::Uuid;
use url::Url as UrlParser;
use std::collections::HashMap;

use super::models::{RawHistoryData, Visit, Url, ExtractionSource};
use super::error::{ExtractionError, Result, FailedFile};

// Safari stores visit timestamps as macOS time (seconds since Jan 1, 2001)
// We need to convert this to Unix time (seconds since Jan 1, 1970)
const MAC_TO_UNIX_EPOCH_OFFSET: i64 = 978307200;

/// Extracts history data from a Safari history.db file
pub fn extract_history(
    file_path: &Path, 
    device_name: Option<String>
) -> Result<RawHistoryData> {
    let path_buf = file_path.to_path_buf();
    
    // Create the container for our extracted data
    let mut history_data = RawHistoryData::new(
        path_buf.clone(),
        device_name
    );
    
    // Open the SQLite database
    let conn = match Connection::open(file_path) {
        Ok(conn) => conn,
        Err(err) => return Err(ExtractionError::Database(
            format!("Failed to open database at {}: {}", file_path.display(), err)
        )),
    };
    
    // First, verify this is a Safari history database
    verify_safari_schema(&conn)?;
    
    // Extract URLs and build a mapping of Safari's IDs to our UUIDs
    let url_id_map = extract_urls(&conn, &mut history_data)?;
    
    // Extract visits using the URL mapping
    extract_visits(&conn, &mut history_data, &url_id_map)?;
    
    Ok(history_data)
}

/// Verifies that the database has the expected Safari history schema
fn verify_safari_schema(conn: &Connection) -> Result<()> {
    // Check for required tables
    let tables = ["history_items", "history_visits"];
    
    for table in tables {
        let query = format!(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='{}'",
            table
        );
        
        let exists: bool = conn.query_row(&query, [], |row| row.get(0))
            .unwrap_or(false);
            
        if !exists {
            return Err(ExtractionError::UnsupportedSchema(
                format!("Not a Safari history database: missing '{}' table", table)
            ));
        }
    }
    
    Ok(())
}

/// Extracts URLs from the history_items table
fn extract_urls(
    conn: &Connection,
    history_data: &mut RawHistoryData
) -> Result<HashMap<i64, Uuid>> {
    let mut url_id_map = HashMap::new();
    
    let query = "
        SELECT id, url, title, domain, visit_count,
               visit_time + 0 as first_visit, 
               last_visited_time + 0 as last_visit
        FROM history_items
    ";
    
    let mut stmt = conn.prepare(query)?;
    let url_rows = stmt.query_map([], |row| Ok(row))?;
    
    for url_result in url_rows {
        let row = url_result?;
        
        // Process each URL
        match process_url_row(row, history_data) {
            Ok(safari_id_uuid_pair) => {
                url_id_map.insert(safari_id_uuid_pair.0, safari_id_uuid_pair.1);
            },
            Err(err) => {
                // Non-fatal error, just add a warning and continue
                history_data.add_warning(&format!("Failed to process URL: {}", err));
            }
        }
    }
    
    Ok(url_id_map)
}

/// Processes a URL row from the database
fn process_url_row(
    row: &Row,
    history_data: &mut RawHistoryData
) -> Result<(i64, Uuid)> {
    let safari_id: i64 = row.get(0)?;
    let url_str: String = row.get(1)?;
    let title: Option<String> = row.get(2)?;
    let domain: String = row.get(3)?;
    
    // Parse timestamps (stored as macOS time)
    let first_visit_mac: i64 = row.get(4)?;
    let last_visit_mac: i64 = row.get(5)?;
    
    // Convert to Unix timestamps and then to UTC DateTime
    let first_seen = mac_to_utc(first_visit_mac)?;
    let last_seen = mac_to_utc(last_visit_mac)?;
    
    // Create a new UUID for this URL
    let url_uuid = Uuid::new_v4();
    
    // Create the URL object
    let url = Url {
        id: url_uuid,
        url: url_str,
        title,
        domain,
        first_seen,
        last_seen,
    };
    
    // Add to our collection
    history_data.urls.push(url);
    
    Ok((safari_id, url_uuid))
}

/// Extracts visits from the history_visits table
fn extract_visits(
    conn: &Connection,
    history_data: &mut RawHistoryData,
    url_id_map: &HashMap<i64, Uuid>
) -> Result<()> {
    let query = "
        SELECT id, history_item, visit_time + 0 as visit_time
        FROM history_visits
        ORDER BY visit_time DESC
    ";
    
    let mut stmt = conn.prepare(query)?;
    let visit_rows = stmt.query_map([], |row| Ok(row))?;
    
    // Get source file name for tracking
    let source_file = history_data.source.file_path.to_string_lossy().to_string();
    
    for visit_result in visit_rows {
        let row = visit_result?;
        
        // Process each visit
        match process_visit_row(row, &source_file, url_id_map, history_data) {
            Ok(_) => {}, // Visit successfully added
            Err(err) => {
                // Non-fatal error, just add a warning and continue
                history_data.add_warning(&format!("Failed to process visit: {}", err));
            }
        }
    }
    
    Ok(())
}

/// Processes a visit row from the database
fn process_visit_row(
    row: &Row,
    source_file: &str,
    url_id_map: &HashMap<i64, Uuid>,
    history_data: &mut RawHistoryData
) -> Result<()> {
    let _visit_id: i64 = row.get(0)?;
    let safari_url_id: i64 = row.get(1)?;
    let visit_time_mac: i64 = row.get(2)?;
    
    // Convert timestamp to UTC
    let visited_at = mac_to_utc(visit_time_mac)?;
    
    // Look up our UUID for this URL
    let url_uuid = match url_id_map.get(&safari_url_id) {
        Some(uuid) => uuid,
        None => return Err(ExtractionError::Parse(
            format!("Visit references unknown URL ID: {}", safari_url_id)
        )),
    };
    
    // Create the Visit object
    let visit = Visit {
        id: Uuid::new_v4(),
        url_id: *url_uuid,
        visited_at,
        visit_count: 1, // Default to 1, we'll aggregate later if needed
        source_file: source_file.to_string(),
        device_name: history_data.source.device_name.clone(),
        duration_sec: None, // Safari doesn't track duration directly
    };
    
    // Add to our collection
    history_data.visits.push(visit);
    
    Ok(())
}

/// Converts a macOS timestamp to UTC DateTime
fn mac_to_utc(mac_timestamp: i64) -> Result<DateTime<Utc>> {
    let unix_timestamp = mac_timestamp + MAC_TO_UNIX_EPOCH_OFFSET;
    
    // Create a UTC datetime
    match Utc.timestamp_opt(unix_timestamp, 0) {
        chrono::offset::LocalResult::Single(dt) => Ok(dt),
        _ => Err(ExtractionError::Parse(
            format!("Invalid timestamp: {}", mac_timestamp)
        )),
    }
}

/// Extracts the domain from a URL
fn extract_domain(url_str: &str) -> Result<String> {
    match UrlParser::parse(url_str) {
        Ok(parsed) => {
            // Get host
            match parsed.host_str() {
                Some(host) => Ok(host.to_string()),
                None => Err(ExtractionError::Parse(
                    format!("URL has no host: {}", url_str)
                )),
            }
        },
        Err(_) => Err(ExtractionError::Parse(
            format!("Invalid URL: {}", url_str)
        )),
    }
}

/// Parses a Safari history.db file and returns the extracted data
/// This is a higher-level function that handles multiple files
pub fn parse_history_db(
    file_paths: &[PathBuf],
    device_names: Option<&[String]>
) -> (Vec<RawHistoryData>, Vec<FailedFile>) {
    let mut successful = Vec::new();
    let mut failed = Vec::new();
    
    for (i, file_path) in file_paths.iter().enumerate() {
        // Get device name if provided
        let device_name = device_names
            .and_then(|names| names.get(i).cloned());
            
        match extract_history(file_path, device_name) {
            Ok(data) => successful.push(data),
            Err(err) => failed.push(FailedFile::new(file_path.clone(), err)),
        }
    }
    
    (successful, failed)
}
