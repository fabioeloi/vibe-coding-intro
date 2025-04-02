// Safari History Knowledge Graph - Main Backend Entry Point

// Import required crates
use tauri::{self, State, command};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::time::Instant;
use std::collections::HashMap;

// Import our modules
mod db;
mod extractor;

// Define app state struct to maintain database connection across commands
struct AppState {
    db_connection: Mutex<Option<db::DatabaseConnection>>,
}

// Processing results returned to the frontend
#[derive(Serialize)]
struct ProcessingResults {
    files_processed: usize,
    urls_processed: usize,
    visits_processed: usize,
    processing_time_sec: f64,
    errors: Vec<String>,
}

// Simplified stats result for frontend
#[derive(Serialize)]
struct HistoryStats {
    url_count: usize,
    visit_count: usize,
    domain_count: usize,
    enriched_count: usize,
    first_visit: Option<String>,
    last_visit: Option<String>,
    top_domains: Vec<(String, usize)>,
}

// Initialize the database
#[command]
async fn initialize_database(app_state: State<'_, AppState>) -> Result<(), String> {
    // Get application data directory
    let app_data_dir = tauri::api::path::app_data_dir(&tauri::Config::default())
        .ok_or_else(|| "Failed to get app data directory".to_string())?;
    
    // Create directories if they don't exist
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    
    // Set database path
    let db_path = app_data_dir.join("history.db");
    
    // Initialize database
    let mut state_guard = app_state.db_connection.lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;
    
    // Create and initialize the database connection
    let connection = db::initialize_database(&db_path)
        .map_err(|e| format!("Failed to initialize database: {}", e))?;
    
    *state_guard = Some(connection);
    
    Ok(())
}

// Process uploaded history files
#[command]
async fn process_history_files(
    file_paths: Vec<String>,
    device_names: Option<Vec<String>>,
    app_state: State<'_, AppState>,
) -> Result<ProcessingResults, String> {
    // Track processing time
    let start_time = Instant::now();
    
    // Convert string paths to PathBuf
    let paths: Vec<PathBuf> = file_paths.iter()
        .map(PathBuf::from)
        .collect();
    
    // Process files with the extractor
    let (successful, failed) = extractor::safari::parse_history_db(
        &paths,
        device_names.as_ref().map(|names| names.as_slice()),
    );
    
    // Collect any errors from failed files
    let mut errors: Vec<String> = failed.iter()
        .map(|f| f.description())
        .collect();
    
    // Ensure we have a database connection
    let state_guard = app_state.db_connection.lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;
    
    let db_conn = state_guard.as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;
    
    // Initialize variables for tracking stats
    let mut total_urls = 0;
    let mut total_visits = 0;
    
    // Insert all successfully processed files into the database
    for history_data in &successful {
        total_urls += history_data.urls.len();
        total_visits += history_data.visits.len();
        
        // Insert the data
        let insert_result = db::operations::insert_history_data(db_conn, history_data)
            .map_err(|e| format!("Database error: {}", e))?;
        
        // Add any insertion errors to the list
        if insert_result.has_errors() {
            errors.extend(insert_result.errors.clone());
        }
    }
    
    // Calculate processing time
    let processing_time = start_time.elapsed().as_secs_f64();
    
    // Return results to the frontend
    Ok(ProcessingResults {
        files_processed: successful.len(),
        urls_processed: total_urls,
        visits_processed: total_visits,
        processing_time_sec: processing_time,
        errors,
    })
}

// Get history statistics
#[command]
async fn get_history_stats(app_state: State<'_, AppState>) -> Result<HistoryStats, String> {
    // Get database connection
    let state_guard = app_state.db_connection.lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;
    
    let db_conn = state_guard.as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;
    
    // Get stats from database
    let stats = db::operations::get_stats(db_conn)
        .map_err(|e| format!("Failed to get stats: {}", e))?;
    
    // Convert timestamps to ISO strings for frontend
    let first_visit = stats.first_visit.map(|dt| dt.to_rfc3339());
    let last_visit = stats.last_visit.map(|dt| dt.to_rfc3339());
    
    // Return formatted stats
    Ok(HistoryStats {
        url_count: stats.url_count,
        visit_count: stats.visit_count,
        domain_count: stats.domain_count,
        enriched_count: stats.enriched_count,
        first_visit,
        last_visit,
        top_domains: stats.top_domains,
    })
}

// Search history
#[command]
async fn search_history(
    query: Option<String>,
    domain: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    app_state: State<'_, AppState>,
) -> Result<Vec<HashMap<String, serde_json::Value>>, String> {
    // Get database connection
    let state_guard = app_state.db_connection.lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;
    
    let db_conn = state_guard.as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;
    
    // Parse date strings to DateTime if provided
    let start = start_date.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));
    let end = end_date.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));
    
    // Set up search parameters
    let search_params = db::operations::SearchParams {
        query,
        domain,
        start_date: start,
        end_date: end,
        limit,
        offset,
    };
    
    // Perform search
    let search_results = db::operations::search_history(db_conn, &search_params)
        .map_err(|e| format!("Search error: {}", e))?;
    
    // Convert results to a format that can be serialized to JSON
    let mut results = Vec::new();
    
    for result in search_results.urls {
        let mut item = HashMap::new();
        
        // Add URL properties
        item.insert("id".to_string(), serde_json::Value::String(result.url.id.to_string()));
        item.insert("url".to_string(), serde_json::Value::String(result.url.url));
        if let Some(title) = result.url.title {
            item.insert("title".to_string(), serde_json::Value::String(title));
        }
        item.insert("domain".to_string(), serde_json::Value::String(result.url.domain));
        item.insert("first_seen".to_string(), serde_json::Value::String(result.url.first_seen.to_rfc3339()));
        item.insert("last_seen".to_string(), serde_json::Value::String(result.url.last_seen.to_rfc3339()));
        item.insert("visit_count".to_string(), serde_json::Value::Number(serde_json::Number::from(result.visit_count)));
        
        // Add metadata if available
        if let Some(metadata) = result.metadata {
            if let Some(summary) = metadata.summary {
                item.insert("summary".to_string(), serde_json::Value::String(summary));
            }
            if let Some(keywords) = metadata.keywords {
                item.insert("keywords".to_string(), serde_json::Value::String(keywords));
            }
            item.insert("is_enriched".to_string(), serde_json::Value::Bool(metadata.is_enriched));
        }
        
        // Add last visit date if available
        if let Some(last_visit) = result.last_visit {
            item.insert("last_visit".to_string(), serde_json::Value::String(last_visit.to_rfc3339()));
        }
        
        results.push(item);
    }
    
    Ok(results)
}

// Get timeline data for visualization
#[command]
async fn get_timeline_data(
    start_date: Option<String>,
    end_date: Option<String>,
    domain: Option<String>,
    group_by: String,
    app_state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    // Get database connection
    let state_guard = app_state.db_connection.lock()
        .map_err(|_| "Failed to acquire database lock".to_string())?;
    
    let db_conn = state_guard.as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;
    
    // Parse date strings to DateTime if provided
    let start = start_date.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));
    let end = end_date.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));
    
    // Timeline parameters for the query
    let timeline_params = db::operations::TimelineParams {
        start_date: start,
        end_date: end,
        domain: domain,
        group_by: match group_by.as_str() {
            "hour" => db::operations::TimelineGrouping::Hour,
            "domain" => db::operations::TimelineGrouping::Domain,
            _ => db::operations::TimelineGrouping::Day, // Default to day
        },
    };
    
    // Call database operation to get timeline data
    let timeline_data = db::operations::get_timeline_data(db_conn, &timeline_params)
        .map_err(|e| format!("Timeline data error: {}", e))?;
    
    // Convert timeline data items to JSON values
    let mut results = Vec::new();
    
    for item in timeline_data {
        let mut data = serde_json::Map::new();
        
        match &item {
            db::operations::TimelineItem::Hourly { hour, count, timestamp, urls } => {
                data.insert("type".to_string(), serde_json::Value::String("hour".to_string()));
                data.insert("hour".to_string(), serde_json::Value::Number(serde_json::Number::from(*hour)));
                data.insert("count".to_string(), serde_json::Value::Number(serde_json::Number::from(*count)));
                data.insert("timestamp".to_string(), serde_json::Value::String(timestamp.to_rfc3339()));
                
                // Add URLs if available
                if let Some(url_list) = urls {
                    let urls_json = serialize_urls(url_list);
                    data.insert("urls".to_string(), urls_json);
                }
            },
            db::operations::TimelineItem::Daily { date, count, urls } => {
                data.insert("type".to_string(), serde_json::Value::String("day".to_string()));
                data.insert("date".to_string(), serde_json::Value::String(date.to_rfc3339()));
                data.insert("count".to_string(), serde_json::Value::Number(serde_json::Number::from(*count)));
                data.insert("timestamp".to_string(), serde_json::Value::String(date.to_rfc3339()));
                
                // Add URLs if available
                if let Some(url_list) = urls {
                    let urls_json = serialize_urls(url_list);
                    data.insert("urls".to_string(), urls_json);
                }
            },
            db::operations::TimelineItem::Domain { domain, count, urls } => {
                data.insert("type".to_string(), serde_json::Value::String("domain".to_string()));
                data.insert("domain".to_string(), serde_json::Value::String(domain.clone()));
                data.insert("count".to_string(), serde_json::Value::Number(serde_json::Number::from(*count)));
                
                // Add URLs if available
                if let Some(url_list) = urls {
                    let urls_json = serialize_urls(url_list);
                    data.insert("urls".to_string(), urls_json);
                }
            },
        }
        
        results.push(serde_json::Value::Object(data));
    }
    
    Ok(results)
}

// Helper function to serialize URL objects to JSON
fn serialize_urls(urls: &[db::models::UrlWithVisits]) -> serde_json::Value {
    let mut url_array = Vec::new();
    
    for url in urls {
        let mut url_obj = serde_json::Map::new();
        
        url_obj.insert("id".to_string(), serde_json::Value::String(url.url.id.to_string()));
        url_obj.insert("url".to_string(), serde_json::Value::String(url.url.url.clone()));
        
        if let Some(ref title) = url.url.title {
            url_obj.insert("title".to_string(), serde_json::Value::String(title.clone()));
        }
        
        url_obj.insert("domain".to_string(), serde_json::Value::String(url.url.domain.clone()));
        url_obj.insert("visit_count".to_string(), serde_json::Value::Number(serde_json::Number::from(url.visit_count)));
        
        if let Some(last_visit) = url.last_visit {
            url_obj.insert("last_visit".to_string(), serde_json::Value::String(last_visit.to_rfc3339()));
        }
        
        url_array.push(serde_json::Value::Object(url_obj));
    }
    
    serde_json::Value::Array(url_array)
}

fn main() {
    // Build Tauri application
    tauri::Builder::default()
        .manage(AppState {
            db_connection: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            initialize_database,
            process_history_files,
            get_history_stats,
            search_history,
            get_timeline_data,
        ])
        .run(tauri::generate_context!())
        .expect("Error running Tauri application");
}
