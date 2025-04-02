// Database Operations
// CRUD operations for history data

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;
use std::collections::HashMap;

use super::error::{DatabaseError, Result};
use super::models::{UrlRecord, VisitRecord, MetadataRecord};
use super::connection::DatabaseConnection;
use crate::extractor::models::RawHistoryData;

/// Inserts extracted history data into the database
pub fn insert_history_data(conn: &DatabaseConnection, history_data: &RawHistoryData) -> Result<InsertStats> {
    let mut stats = InsertStats::default();
    
    // Use a transaction for better performance and atomicity
    conn.transaction(|tx| {
        // First, insert all URLs
        for url in &history_data.urls {
            match insert_url(tx, &UrlRecord {
                id: url.id,
                url: url.url.clone(),
                title: url.title.clone(),
                domain: url.domain.clone(),
                first_seen: url.first_seen,
                last_seen: url.last_seen,
            }) {
                Ok(_) => stats.urls_inserted += 1,
                Err(e) => {
                    stats.errors.push(format!("Failed to insert URL {}: {}", url.url, e));
                    continue; // Skip visits for this URL
                }
            }
            
            // Insert empty metadata record
            match insert_metadata(tx, &MetadataRecord::empty(url.id)) {
                Ok(_) => stats.metadata_inserted += 1,
                Err(e) => {
                    stats.errors.push(format!("Failed to insert metadata for URL {}: {}", url.url, e));
                }
            }
        }
        
        // Then, insert all visits
        for visit in &history_data.visits {
            match insert_visit(tx, &VisitRecord {
                id: visit.id,
                url_id: visit.url_id,
                visited_at: visit.visited_at,
                visit_count: visit.visit_count,
                source_file: visit.source_file.clone(),
                device_name: visit.device_name.clone(),
                duration_sec: visit.duration_sec,
            }) {
                Ok(_) => stats.visits_inserted += 1,
                Err(e) => {
                    stats.errors.push(format!("Failed to insert visit {}: {}", visit.id, e));
                }
            }
        }
        
        Ok(stats)
    })
}

/// Inserts a URL record into the database
fn insert_url(conn: &Connection, url: &UrlRecord) -> Result<()> {
    // Check if URL already exists (by URL string)
    let existing = conn.query_row(
        "SELECT id FROM url WHERE url = ?",
        [&url.url],
        |row| {
            let id_str: String = row.get(0)?;
            Ok(id_str)
        },
    );
    
    match existing {
        Ok(_) => {
            // URL exists, update last_seen time if newer
            conn.execute(
                "UPDATE url SET last_seen = MAX(last_seen, ?) WHERE url = ?",
                params![url.last_seen.timestamp(), url.url],
            ).map_err(|e| DatabaseError::Query(e.to_string()))?;
        },
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // URL doesn't exist, insert it
            conn.execute(
                "INSERT INTO url (id, url, title, domain, first_seen, last_seen)
                 VALUES (?, ?, ?, ?, ?, ?)",
                url.to_params(),
            ).map_err(|e| DatabaseError::Query(e.to_string()))?;
        },
        Err(e) => return Err(DatabaseError::Query(e.to_string())),
    }
    
    Ok(())
}

/// Inserts a visit record into the database
fn insert_visit(conn: &Connection, visit: &VisitRecord) -> Result<()> {
    // Check if the exact same visit already exists
    let existing = conn.query_row(
        "SELECT id FROM visit WHERE url_id = ? AND visited_at = ? AND source_file = ?",
        params![visit.url_id.to_string(), visit.visited_at.timestamp(), visit.source_file],
        |row| {
            let id_str: String = row.get(0)?;
            Ok(id_str)
        },
    );
    
    match existing {
        Ok(_) => {
            // Visit already exists, skip
            return Ok(());
        },
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // Visit doesn't exist, insert it
            conn.execute(
                "INSERT INTO visit (id, url_id, visited_at, visit_count, source_file, device_name, duration_sec)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
                visit.to_params(),
            ).map_err(|e| DatabaseError::Query(e.to_string()))?;
        },
        Err(e) => return Err(DatabaseError::Query(e.to_string())),
    }
    
    Ok(())
}

/// Inserts a metadata record into the database
fn insert_metadata(conn: &Connection, metadata: &MetadataRecord) -> Result<()> {
    // Check if metadata for this URL already exists
    let existing = conn.query_row(
        "SELECT url_id FROM metadata WHERE url_id = ?",
        [metadata.url_id.to_string()],
        |row| {
            let id_str: String = row.get(0)?;
            Ok(id_str)
        },
    );
    
    match existing {
        Ok(_) => {
            // Metadata exists, only update if we have enrichment
            if metadata.is_enriched {
                conn.execute(
                    "UPDATE metadata SET summary = ?, keywords = ?, tags = ?, 
                     topic_cluster = ?, is_enriched = ?
                     WHERE url_id = ?",
                    params![
                        metadata.summary,
                        metadata.keywords,
                        metadata.tags,
                        metadata.topic_cluster,
                        metadata.is_enriched,
                        metadata.url_id.to_string()
                    ],
                ).map_err(|e| DatabaseError::Query(e.to_string()))?;
            }
        },
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // Metadata doesn't exist, insert it
            conn.execute(
                "INSERT INTO metadata (url_id, summary, keywords, tags, topic_cluster, is_enriched)
                 VALUES (?, ?, ?, ?, ?, ?)",
                metadata.to_params(),
            ).map_err(|e| DatabaseError::Query(e.to_string()))?;
        },
        Err(e) => return Err(DatabaseError::Query(e.to_string())),
    }
    
    Ok(())
}

/// Statistics for inserted records
#[derive(Debug, Default)]
pub struct InsertStats {
    /// Number of URLs inserted
    pub urls_inserted: usize,
    /// Number of visits inserted
    pub visits_inserted: usize,
    /// Number of metadata records inserted
    pub metadata_inserted: usize,
    /// Any errors that occurred during insertion
    pub errors: Vec<String>,
}

impl InsertStats {
    /// Returns the total number of records inserted
    pub fn total_inserted(&self) -> usize {
        self.urls_inserted + self.visits_inserted + self.metadata_inserted
    }
    
    /// Returns true if there were any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Parameters for searching history
pub struct SearchParams {
    /// Text to search for (in URL, title, or summary)
    pub query: Option<String>,
    /// Filter by domain
    pub domain: Option<String>,
    /// Start date range
    pub start_date: Option<DateTime<Utc>>,
    /// End date range
    pub end_date: Option<DateTime<Utc>>,
    /// Limit number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Results from a history search
pub struct SearchResults {
    /// The URLs found
    pub urls: Vec<SearchResult>,
    /// Total number of matches (may be more than returned due to limit)
    pub total_count: usize,
}

/// A single search result
pub struct SearchResult {
    /// The URL record
    pub url: UrlRecord,
    /// Optional metadata for the URL
    pub metadata: Option<MetadataRecord>,
    /// Count of visits to this URL
    pub visit_count: usize,
    /// Most recent visit
    pub last_visit: Option<DateTime<Utc>>,
}

/// Searches history based on the given parameters
pub fn search_history(conn: &DatabaseConnection, params: &SearchParams) -> Result<SearchResults> {
    conn.with_connection(|tx| {
        // Build the query based on the parameters
        let mut query = String::from(
            "SELECT u.id, u.url, u.title, u.domain, u.first_seen, u.last_seen,
                    COUNT(v.id) as visit_count,
                    MAX(v.visited_at) as last_visit
             FROM url u
             LEFT JOIN visit v ON u.id = v.url_id"
        );
        
        let mut where_clauses = Vec::new();
        let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        
        // Add search conditions
        if let Some(q) = &params.query {
            where_clauses.push(
                "(u.url LIKE ? OR u.title LIKE ? OR EXISTS (
                    SELECT 1 FROM metadata m 
                    WHERE m.url_id = u.id AND (
                        m.summary LIKE ? OR 
                        m.keywords LIKE ? OR 
                        m.tags LIKE ?
                    )
                ))".to_string()
            );
            
            let like_pattern = format!("%{}%", q);
            query_params.push(Box::new(like_pattern.clone()));
            query_params.push(Box::new(like_pattern.clone()));
            query_params.push(Box::new(like_pattern.clone()));
            query_params.push(Box::new(like_pattern.clone()));
            query_params.push(Box::new(like_pattern));
        }
        
        if let Some(domain) = &params.domain {
            where_clauses.push("u.domain = ?".to_string());
            query_params.push(Box::new(domain.clone()));
        }
        
        if let Some(start) = params.start_date {
            where_clauses.push("v.visited_at >= ?".to_string());
            query_params.push(Box::new(start.timestamp()));
        }
        
        if let Some(end) = params.end_date {
            where_clauses.push("v.visited_at <= ?".to_string());
            query_params.push(Box::new(end.timestamp()));
        }
        
        // Add WHERE clause if we have conditions
        if !where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&where_clauses.join(" AND "));
        }
        
        // Add GROUP BY and ORDER BY
        query.push_str(" GROUP BY u.id ORDER BY last_visit DESC");
        
        // Add LIMIT and OFFSET
        if let Some(limit) = params.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        
        if let Some(offset) = params.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }
        
        // Execute the query
        let mut stmt = tx.prepare(&query)?;
        
        let url_rows = stmt.query_map(rusqlite::params_from_iter(query_params.iter().map(|p| p.as_ref())), |row| {
            let url = UrlRecord::from_row(row)?;
            let visit_count: i64 = row.get(6)?;
            let last_visit_ts: Option<i64> = row.get(7)?;
            
            let last_visit = last_visit_ts.map(|ts| {
                DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now())
            });
            
            Ok((url, visit_count as usize, last_visit))
        })?;
        
        // Collect results
        let mut urls = Vec::new();
        for row_result in url_rows {
            let (url, visit_count, last_visit) = row_result?;
            
            // Get metadata for this URL
            let metadata = get_metadata_for_url(tx, url.id)?;
            
            urls.push(SearchResult {
                url,
                metadata,
                visit_count,
                last_visit,
            });
        }
        
        // Get total count (without limit/offset)
        let total_count = if params.limit.is_some() || params.offset.is_some() {
            // Build count query with same WHERE clauses
            let mut count_query = String::from("SELECT COUNT(DISTINCT u.id) FROM url u LEFT JOIN visit v ON u.id = v.url_id");
            
            if !where_clauses.is_empty() {
                count_query.push_str(" WHERE ");
                count_query.push_str(&where_clauses.join(" AND "));
            }
            
            let count: i64 = tx.query_row(
                &count_query,
                rusqlite::params_from_iter(query_params.iter().map(|p| p.as_ref())),
                |row| row.get(0),
            )?;
            
            count as usize
        } else {
            urls.len()
        };
        
        Ok(SearchResults {
            urls,
            total_count,
        })
    })
}

/// Gets metadata for a URL
fn get_metadata_for_url(conn: &Connection, url_id: Uuid) -> Result<Option<MetadataRecord>> {
    match conn.query_row(
        "SELECT url_id, summary, keywords, tags, topic_cluster, is_enriched
         FROM metadata WHERE url_id = ?",
        [url_id.to_string()],
        |row| MetadataRecord::from_row(row),
    ) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::Query(e.to_string())),
    }
}

/// Statistics about the browsing history
pub struct HistoryStats {
    /// Total number of URLs
    pub url_count: usize,
    /// Total number of visits
    pub visit_count: usize,
    /// Number of unique domains
    pub domain_count: usize,
    /// Date of first visit
    pub first_visit: Option<DateTime<Utc>>,
    /// Date of most recent visit
    pub last_visit: Option<DateTime<Utc>>,
    /// Number of enriched URLs (with AI metadata)
    pub enriched_count: usize,
    /// Top domains by visit count
    pub top_domains: Vec<(String, usize)>,
}

/// Gets statistics about the browsing history
pub fn get_stats(conn: &DatabaseConnection) -> Result<HistoryStats> {
    conn.with_connection(|tx| {
        // Get URL and visit counts
        let url_count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM url",
            [],
            |row| row.get(0),
        )?;
        
        let visit_count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM visit",
            [],
            |row| row.get(0),
        )?;
        
        let domain_count: i64 = tx.query_row(
            "SELECT COUNT(DISTINCT domain) FROM url",
            [],
            |row| row.get(0),
        )?;
        
        let enriched_count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM metadata WHERE is_enriched = 1",
            [],
            |row| row.get(0),
        )?;
        
        // Get first and last visit times
        let first_visit_ts: Option<i64> = tx.query_row(
            "SELECT MIN(visited_at) FROM visit",
            [],
            |row| row.get(0),
        )?;
        
        let last_visit_ts: Option<i64> = tx.query_row(
            "SELECT MAX(visited_at) FROM visit",
            [],
            |row| row.get(0),
        )?;
        
        let first_visit = first_visit_ts.map(|ts| {
            DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now())
        });
        
        let last_visit = last_visit_ts.map(|ts| {
            DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now())
        });
        
        // Get top domains
        let mut stmt = tx.prepare(
            "SELECT domain, COUNT(*) as count
             FROM url u
             JOIN visit v ON u.id = v.url_id
             GROUP BY domain
             ORDER BY count DESC
             LIMIT 10"
        )?;
        
        let domain_rows = stmt.query_map([], |row| {
            let domain: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((domain, count as usize))
        })?;
        
        let mut top_domains = Vec::new();
        for row_result in domain_rows {
            top_domains.push(row_result?);
        }
        
        Ok(HistoryStats {
            url_count: url_count as usize,
            visit_count: visit_count as usize,
            domain_count: domain_count as usize,
            first_visit,
            last_visit,
            enriched_count: enriched_count as usize,
            top_domains,
        })
    })
}

/// Parameters for timeline data visualization
pub struct TimelineParams {
    /// Start date for timeline data
    pub start_date: Option<DateTime<Utc>>,
    /// End date for timeline data
    pub end_date: Option<DateTime<Utc>>,
    /// Optional domain filter
    pub domain: Option<String>,
    /// How to group the timeline data
    pub group_by: TimelineGrouping,
}

/// Grouping type for timeline visualization
pub enum TimelineGrouping {
    /// Group by hour of day
    Hour,
    /// Group by day
    Day,
    /// Group by domain
    Domain,
}

/// Timeline data item, variant depends on grouping type
pub enum TimelineItem {
    /// Hourly grouping item
    Hourly {
        /// Hour of day (0-23)
        hour: u8,
        /// Number of visits in this hour
        count: u32,
        /// Timestamp for this hour (for display purposes)
        timestamp: DateTime<Utc>,
        /// Optional sample of URLs visited in this hour
        urls: Option<Vec<crate::db::models::UrlWithVisits>>,
    },
    /// Daily grouping item
    Daily {
        /// Date for this day
        date: DateTime<Utc>,
        /// Number of visits on this day
        count: u32,
        /// Optional sample of URLs visited on this day
        urls: Option<Vec<crate::db::models::UrlWithVisits>>,
    },
    /// Domain grouping item
    Domain {
        /// Domain name
        domain: String,
        /// Number of visits to this domain
        count: u32,
        /// Optional sample of URLs for this domain
        urls: Option<Vec<crate::db::models::UrlWithVisits>>,
    },
}

/// Gets timeline data based on the given parameters
pub fn get_timeline_data(
    conn: &DatabaseConnection,
    params: &TimelineParams,
) -> Result<Vec<TimelineItem>> {
    // Use existing connection to perform query
    conn.use_connection(|c| match params.group_by {
        TimelineGrouping::Hour => get_hourly_timeline_data(c, params),
        TimelineGrouping::Day => get_daily_timeline_data(c, params),
        TimelineGrouping::Domain => get_domain_timeline_data(c, params),
    })
}

/// Gets timeline data grouped by hour of day
fn get_hourly_timeline_data(
    conn: &Connection,
    params: &TimelineParams,
) -> Result<Vec<TimelineItem>> {
    // Build query to group visits by hour of day
    let mut query = String::from(
        "SELECT 
            strftime('%H', datetime(visited_at, 'unixepoch')) as hour,
            COUNT(*) as count,
            MIN(visited_at) as sample_timestamp
         FROM visit
         JOIN url ON visit.url_id = url.id"
    );
    
    // Add WHERE clauses for filters
    let mut conditions = Vec::new();
    let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    
    if let Some(ref start_date) = params.start_date {
        conditions.push("visited_at >= ?");
        query_params.push(Box::new(start_date.timestamp()));
    }
    
    if let Some(ref end_date) = params.end_date {
        conditions.push("visited_at <= ?");
        query_params.push(Box::new(end_date.timestamp()));
    }
    
    if let Some(ref domain) = params.domain {
        conditions.push("url.domain = ?");
        query_params.push(Box::new(domain.clone()));
    }
    
    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }
    
    // Group by hour and order by visit count
    query.push_str(" GROUP BY hour ORDER BY count DESC, hour ASC");
    
    // Execute query
    let mut stmt = conn.prepare(&query)?;
    
    let results = stmt.query_map(rusqlite::params_from_iter(query_params.iter().map(|p| p.as_ref())), |row| {
        let hour_str: String = row.get(0)?;
        let count: u32 = row.get(1)?;
        let timestamp: i64 = row.get(2)?;
        
        // Parse hour
        let hour: u8 = hour_str.parse().unwrap_or(0);
        
        // Create timestamp for display
        let timestamp = DateTime::from_timestamp(timestamp, 0)
            .unwrap_or_else(|| Utc::now());
        
        Ok(TimelineItem::Hourly {
            hour,
            count,
            timestamp,
            urls: None, // We'll fill this in separately
        })
    })?;
    
    let mut timeline_items = Vec::new();
    for result in results {
        timeline_items.push(result?);
    }
    
    // Fetch sample URLs for each hour (if timeline items exist)
    if !timeline_items.is_empty() {
        fetch_sample_urls_for_timeline(&mut timeline_items, conn, params)?;
    }
    
    Ok(timeline_items)
}

/// Gets timeline data grouped by day
fn get_daily_timeline_data(
    conn: &Connection,
    params: &TimelineParams,
) -> Result<Vec<TimelineItem>> {
    // Build query to group visits by day
    let mut query = String::from(
        "SELECT 
            strftime('%Y-%m-%d', datetime(visited_at, 'unixepoch')) as day,
            COUNT(*) as count,
            MIN(visited_at) as sample_timestamp
         FROM visit
         JOIN url ON visit.url_id = url.id"
    );
    
    // Add WHERE clauses for filters
    let mut conditions = Vec::new();
    let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    
    if let Some(ref start_date) = params.start_date {
        conditions.push("visited_at >= ?");
        query_params.push(Box::new(start_date.timestamp()));
    }
    
    if let Some(ref end_date) = params.end_date {
        conditions.push("visited_at <= ?");
        query_params.push(Box::new(end_date.timestamp()));
    }
    
    if let Some(ref domain) = params.domain {
        conditions.push("url.domain = ?");
        query_params.push(Box::new(domain.clone()));
    }
    
    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }
    
    // Group by day and order by date (newest first)
    query.push_str(" GROUP BY day ORDER BY day DESC");
    
    // Execute query
    let mut stmt = conn.prepare(&query)?;
    
    let results = stmt.query_map(rusqlite::params_from_iter(query_params.iter().map(|p| p.as_ref())), |row| {
        let _day_str: String = row.get(0)?;
        let count: u32 = row.get(1)?;
        let timestamp: i64 = row.get(2)?;
        
        // Create date for this day
        let date = DateTime::from_timestamp(timestamp, 0)
            .unwrap_or_else(|| Utc::now());
        
        Ok(TimelineItem::Daily {
            date,
            count,
            urls: None, // We'll fill this in separately
        })
    })?;
    
    let mut timeline_items = Vec::new();
    for result in results {
        timeline_items.push(result?);
    }
    
    // Fetch sample URLs for each day (if timeline items exist)
    if !timeline_items.is_empty() {
        fetch_sample_urls_for_timeline(&mut timeline_items, conn, params)?;
    }
    
    Ok(timeline_items)
}

/// Gets timeline data grouped by domain
fn get_domain_timeline_data(
    conn: &Connection,
    params: &TimelineParams,
) -> Result<Vec<TimelineItem>> {
    // Build query to group visits by domain
    let mut query = String::from(
        "SELECT 
            url.domain,
            COUNT(*) as count
         FROM visit
         JOIN url ON visit.url_id = url.id"
    );
    
    // Add WHERE clauses for filters
    let mut conditions = Vec::new();
    let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    
    if let Some(ref start_date) = params.start_date {
        conditions.push("visited_at >= ?");
        query_params.push(Box::new(start_date.timestamp()));
    }
    
    if let Some(ref end_date) = params.end_date {
        conditions.push("visited_at <= ?");
        query_params.push(Box::new(end_date.timestamp()));
    }
    
    if let Some(ref domain) = params.domain {
        conditions.push("url.domain = ?");
        query_params.push(Box::new(domain.clone()));
    }
    
    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }
    
    // Group by domain and order by visit count
    query.push_str(" GROUP BY url.domain ORDER BY count DESC LIMIT 100");
    
    // Execute query
    let mut stmt = conn.prepare(&query)?;
    
    let results = stmt.query_map(rusqlite::params_from_iter(query_params.iter().map(|p| p.as_ref())), |row| {
        let domain: String = row.get(0)?;
        let count: u32 = row.get(1)?;
        
        Ok(TimelineItem::Domain {
            domain,
            count,
            urls: None, // We'll fill this in separately
        })
    })?;
    
    let mut timeline_items = Vec::new();
    for result in results {
        timeline_items.push(result?);
    }
    
    // Fetch sample URLs for each domain (if timeline items exist)
    if !timeline_items.is_empty() {
        fetch_sample_urls_for_timeline(&mut timeline_items, conn, params)?;
    }
    
    Ok(timeline_items)
}

/// Helper function to fetch sample URLs for timeline items
fn fetch_sample_urls_for_timeline(
    timeline_items: &mut Vec<TimelineItem>,
    conn: &Connection,
    params: &TimelineParams,
) -> Result<()> {
    // For each timeline item, fetch a sample of URLs
    for item in timeline_items.iter_mut() {
        match item {
            TimelineItem::Hourly { hour, timestamp, urls, .. } => {
                // Fetch sample URLs for this hour
                let mut query = String::from(
                    "SELECT url.id, url.url, url.title, url.domain, 
                     COUNT(visit.id) as visit_count,
                     MAX(visit.visited_at) as last_visit
                     FROM visit
                     JOIN url ON visit.url_id = url.id
                     WHERE strftime('%H', datetime(visited_at, 'unixepoch')) = ?"
                );
                
                let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
                query_params.push(Box::new(format!("{:02}", hour)));
                
                if let Some(ref start_date) = params.start_date {
                    query.push_str(" AND visited_at >= ?");
                    query_params.push(Box::new(start_date.timestamp()));
                }
                
                if let Some(ref end_date) = params.end_date {
                    query.push_str(" AND visited_at <= ?");
                    query_params.push(Box::new(end_date.timestamp()));
                }
                
                if let Some(ref domain) = params.domain {
                    query.push_str(" AND url.domain = ?");
                    query_params.push(Box::new(domain.clone()));
                }
                
                query.push_str(" GROUP BY url.id ORDER BY visit_count DESC LIMIT 5");
                
                *urls = Some(fetch_urls_with_query(conn, &query, &query_params)?);
            },
            TimelineItem::Daily { date, urls, .. } => {
                // Extract day string from the date
                let day_str = date.format("%Y-%m-%d").to_string();
                
                // Fetch sample URLs for this day
                let mut query = String::from(
                    "SELECT url.id, url.url, url.title, url.domain, 
                     COUNT(visit.id) as visit_count,
                     MAX(visit.visited_at) as last_visit
                     FROM visit
                     JOIN url ON visit.url_id = url.id
                     WHERE strftime('%Y-%m-%d', datetime(visited_at, 'unixepoch')) = ?"
                );
                
                let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
                query_params.push(Box::new(day_str));
                
                if let Some(ref domain) = params.domain {
                    query.push_str(" AND url.domain = ?");
                    query_params.push(Box::new(domain.clone()));
                }
                
                query.push_str(" GROUP BY url.id ORDER BY visit_count DESC LIMIT 5");
                
                *urls = Some(fetch_urls_with_query(conn, &query, &query_params)?);
            },
            TimelineItem::Domain { domain, urls, .. } => {
                // Fetch sample URLs for this domain
                let mut query = String::from(
                    "SELECT url.id, url.url, url.title, url.domain, 
                     COUNT(visit.id) as visit_count,
                     MAX(visit.visited_at) as last_visit
                     FROM visit
                     JOIN url ON visit.url_id = url.id
                     WHERE url.domain = ?"
                );
                
                let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
                query_params.push(Box::new(domain.clone()));
                
                if let Some(ref start_date) = params.start_date {
                    query.push_str(" AND visited_at >= ?");
                    query_params.push(Box::new(start_date.timestamp()));
                }
                
                if let Some(ref end_date) = params.end_date {
                    query.push_str(" AND visited_at <= ?");
                    query_params.push(Box::new(end_date.timestamp()));
                }
                
                query.push_str(" GROUP BY url.id ORDER BY visit_count DESC LIMIT 5");
                
                *urls = Some(fetch_urls_with_query(conn, &query, &query_params)?);
            },
        }
    }
    
    Ok(())
}

/// Helper function to fetch URLs with a given query
fn fetch_urls_with_query(
    conn: &Connection,
    query: &str,
    params: &[Box<dyn rusqlite::ToSql>],
) -> Result<Vec<crate::db::models::UrlWithVisits>> {
    let mut stmt = conn.prepare(query)?;
    
    let url_iter = stmt.query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), |row| {
        let id_str: String = row.get(0)?;
        let url: String = row.get(1)?;
        let title: Option<String> = row.get(2)?;
        let domain: String = row.get(3)?;
        let visit_count: i32 = row.get(4)?;
        let last_visit_ts: Option<i64> = row.get(5)?;
        
        // Parse UUID from string
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| rusqlite::Error::InvalidColumnType(0, format!("Invalid UUID: {}", e)))?;
        
        // Convert timestamp to DateTime if available
        let last_visit = last_visit_ts.map(|ts| {
            DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now())
        });
        
        Ok(crate::db::models::UrlWithVisits {
            url: crate::db::models::UrlRecord {
                id,
                url,
                title,
                domain,
                first_seen: Utc::now(), // Not used in this context
                last_seen: Utc::now(),  // Not used in this context
            },
            visit_count: visit_count as usize,
            last_visit,
        })
    })?;
    
    let mut urls = Vec::new();
    for url_result in url_iter {
        urls.push(url_result?);
    }
    
    Ok(urls)
}
