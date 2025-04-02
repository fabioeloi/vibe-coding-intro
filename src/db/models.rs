// Database Models
// ORM-like data models for database entities

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use rusqlite::{Row, params, Statement};
use std::convert::TryFrom;

use super::error::{DatabaseError, Result};

/// Represents a URL record in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlRecord {
    /// Unique identifier
    pub id: Uuid,
    /// The complete URL
    pub url: String,
    /// Page title
    pub title: Option<String>,
    /// Domain name (e.g., example.com)
    pub domain: String,
    /// When the URL was first seen
    pub first_seen: DateTime<Utc>,
    /// When the URL was last seen
    pub last_seen: DateTime<Utc>,
}

/// Represents a visit record in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisitRecord {
    /// Unique identifier
    pub id: Uuid,
    /// Reference to the URL that was visited
    pub url_id: Uuid,
    /// When the visit occurred
    pub visited_at: DateTime<Utc>,
    /// Visit count
    pub visit_count: i32,
    /// Source file this visit was extracted from
    pub source_file: String,
    /// Optional device name
    pub device_name: Option<String>,
    /// Optional visit duration in seconds
    pub duration_sec: Option<f64>,
}

/// Represents a metadata record in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataRecord {
    /// URL ID this metadata belongs to
    pub url_id: Uuid,
    /// Summary of the page content
    pub summary: Option<String>,
    /// Keywords extracted from the content (JSON array as string)
    pub keywords: Option<String>,
    /// User or AI assigned tags (JSON array as string)
    pub tags: Option<String>,
    /// Topic cluster assignment
    pub topic_cluster: Option<String>,
    /// Whether this URL has been enriched with AI
    pub is_enriched: bool,
}

// Implementation for UrlRecord
impl UrlRecord {
    /// Creates a new URL record
    pub fn new(
        url: String,
        title: Option<String>,
        domain: String,
        first_seen: DateTime<Utc>,
        last_seen: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            url,
            title,
            domain,
            first_seen,
            last_seen,
        }
    }
    
    /// Converts this record to SQLite parameters for insertion
    pub fn to_params(&self) -> [&dyn rusqlite::ToSql; 6] {
        [
            &self.id.to_string(),
            &self.url,
            &self.title,
            &self.domain,
            &self.first_seen.timestamp(),
            &self.last_seen.timestamp(),
        ]
    }
    
    /// Creates a record from a SQLite row
    pub fn from_row(row: &Row) -> Result<Self> {
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| DatabaseError::Data(format!("Invalid UUID: {}", e)))?;
            
        let url: String = row.get(1)?;
        let title: Option<String> = row.get(2)?;
        let domain: String = row.get(3)?;
        
        let first_seen_ts: i64 = row.get(4)?;
        let last_seen_ts: i64 = row.get(5)?;
        
        let first_seen = DateTime::from_timestamp(first_seen_ts, 0)
            .ok_or_else(|| DatabaseError::Data(format!("Invalid timestamp: {}", first_seen_ts)))?;
            
        let last_seen = DateTime::from_timestamp(last_seen_ts, 0)
            .ok_or_else(|| DatabaseError::Data(format!("Invalid timestamp: {}", last_seen_ts)))?;
            
        Ok(Self {
            id,
            url,
            title,
            domain,
            first_seen,
            last_seen,
        })
    }
}

// Implementation for VisitRecord
impl VisitRecord {
    /// Creates a new visit record
    pub fn new(
        url_id: Uuid,
        visited_at: DateTime<Utc>,
        visit_count: i32,
        source_file: String,
        device_name: Option<String>,
        duration_sec: Option<f64>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            url_id,
            visited_at,
            visit_count,
            source_file,
            device_name,
            duration_sec,
        }
    }
    
    /// Converts this record to SQLite parameters for insertion
    pub fn to_params(&self) -> [&dyn rusqlite::ToSql; 7] {
        [
            &self.id.to_string(),
            &self.url_id.to_string(),
            &self.visited_at.timestamp(),
            &self.visit_count,
            &self.source_file,
            &self.device_name,
            &self.duration_sec,
        ]
    }
    
    /// Creates a record from a SQLite row
    pub fn from_row(row: &Row) -> Result<Self> {
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| DatabaseError::Data(format!("Invalid UUID: {}", e)))?;
            
        let url_id_str: String = row.get(1)?;
        let url_id = Uuid::parse_str(&url_id_str)
            .map_err(|e| DatabaseError::Data(format!("Invalid URL ID: {}", e)))?;
            
        let visited_at_ts: i64 = row.get(2)?;
        let visited_at = DateTime::from_timestamp(visited_at_ts, 0)
            .ok_or_else(|| DatabaseError::Data(format!("Invalid timestamp: {}", visited_at_ts)))?;
            
        let visit_count: i32 = row.get(3)?;
        let source_file: String = row.get(4)?;
        let device_name: Option<String> = row.get(5)?;
        let duration_sec: Option<f64> = row.get(6)?;
            
        Ok(Self {
            id,
            url_id,
            visited_at,
            visit_count,
            source_file,
            device_name,
            duration_sec,
        })
    }
}

// Implementation for MetadataRecord
impl MetadataRecord {
    /// Creates a new metadata record
    pub fn new(
        url_id: Uuid,
        summary: Option<String>,
        keywords: Option<String>,
        tags: Option<String>,
        topic_cluster: Option<String>,
        is_enriched: bool,
    ) -> Self {
        Self {
            url_id,
            summary,
            keywords,
            tags,
            topic_cluster,
            is_enriched,
        }
    }
    
    /// Creates an empty metadata record for a URL
    pub fn empty(url_id: Uuid) -> Self {
        Self {
            url_id,
            summary: None,
            keywords: None,
            tags: None,
            topic_cluster: None,
            is_enriched: false,
        }
    }
    
    /// Converts this record to SQLite parameters for insertion
    pub fn to_params(&self) -> [&dyn rusqlite::ToSql; 6] {
        [
            &self.url_id.to_string(),
            &self.summary,
            &self.keywords,
            &self.tags,
            &self.topic_cluster,
            &self.is_enriched,
        ]
    }
    
    /// Creates a record from a SQLite row
    pub fn from_row(row: &Row) -> Result<Self> {
        let url_id_str: String = row.get(0)?;
        let url_id = Uuid::parse_str(&url_id_str)
            .map_err(|e| DatabaseError::Data(format!("Invalid URL ID: {}", e)))?;
            
        let summary: Option<String> = row.get(1)?;
        let keywords: Option<String> = row.get(2)?;
        let tags: Option<String> = row.get(3)?;
        let topic_cluster: Option<String> = row.get(4)?;
        let is_enriched: bool = row.get(5)?;
            
        Ok(Self {
            url_id,
            summary,
            keywords,
            tags,
            topic_cluster,
            is_enriched,
        })
    }
}
