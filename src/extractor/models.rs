// Safari History Extractor - Data Models
// Defines the data structures used for extracting and storing Safari history data

use std::path::PathBuf;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Represents a visit to a URL extracted from Safari history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visit {
    /// Unique identifier for this visit
    pub id: Uuid,
    /// Reference to the URL that was visited
    pub url_id: Uuid,
    /// When the visit occurred
    pub visited_at: DateTime<Utc>,
    /// How many times this URL was visited in this session
    pub visit_count: i32,
    /// Source file where this visit was extracted from
    pub source_file: String,
    /// Optional name of the device
    pub device_name: Option<String>,
    /// Optional duration of the visit in seconds
    pub duration_sec: Option<f64>,
}

/// Represents a URL from the Safari history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Url {
    /// Unique identifier for this URL
    pub id: Uuid,
    /// The full URL string
    pub url: String,
    /// Title of the page, if available
    pub title: Option<String>,
    /// Extracted domain from the URL
    pub domain: String,
    /// When this URL was first seen
    pub first_seen: DateTime<Utc>,
    /// When this URL was last seen
    pub last_seen: DateTime<Utc>,
}

/// Information about the source of the extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionSource {
    /// Path to the history.db file
    pub file_path: PathBuf,
    /// Optional name to identify the device
    pub device_name: Option<String>,
    /// When the extraction was performed
    pub extraction_time: DateTime<Utc>,
}

/// Container for the raw data extracted from a history.db file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawHistoryData {
    /// Information about the source of this data
    pub source: ExtractionSource,
    /// List of URLs extracted
    pub urls: Vec<Url>,
    /// List of visits extracted
    pub visits: Vec<Visit>,
    /// Any warnings or non-fatal issues encountered during extraction
    pub warnings: Vec<String>,
}

/// Implements utility methods for RawHistoryData
impl RawHistoryData {
    /// Creates a new empty RawHistoryData with the specified source
    pub fn new(file_path: PathBuf, device_name: Option<String>) -> Self {
        Self {
            source: ExtractionSource {
                file_path,
                device_name,
                extraction_time: Utc::now(),
            },
            urls: Vec::new(),
            visits: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Adds a warning message to the extraction
    pub fn add_warning(&mut self, message: &str) {
        self.warnings.push(message.to_string());
    }

    /// Returns the total number of items (URLs + visits) in this extraction
    pub fn total_items(&self) -> usize {
        self.urls.len() + self.visits.len()
    }
}
