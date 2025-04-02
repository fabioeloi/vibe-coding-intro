// Safari History Extractor Module
// This module handles parsing and extracting data from Safari history.db files

// We'll organize this module into:
// - safari.rs: Safari-specific parsing logic
// - models.rs: Data models for extraction
// - error.rs: Error handling

pub mod safari;
pub mod models;
pub mod error;

pub use safari::{extract_history, parse_history_db};
pub use models::{Visit, Url, RawHistoryData, ExtractionSource};
pub use error::ExtractionError;
