#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use rusqlite::Connection;

    // Helper function to create a mock Safari history.db for testing
    fn create_mock_safari_db() -> (PathBuf, Connection) {
        let dir = tempdir().expect("Failed to create temp directory");
        let db_path = dir.path().join("history.db");
        
        // Create and open the database
        let conn = Connection::open(&db_path).expect("Failed to create mock database");
        
        // Create the tables matching Safari's schema
        conn.execute(
            "CREATE TABLE history_items (
                id INTEGER PRIMARY KEY,
                url TEXT NOT NULL,
                title TEXT,
                domain TEXT NOT NULL,
                visit_count INTEGER,
                visit_time INTEGER,
                last_visited_time INTEGER
            )",
            [],
        ).expect("Failed to create history_items table");
        
        conn.execute(
            "CREATE TABLE history_visits (
                id INTEGER PRIMARY KEY,
                history_item INTEGER NOT NULL,
                visit_time INTEGER NOT NULL,
                FOREIGN KEY(history_item) REFERENCES history_items(id)
            )",
            [],
        ).expect("Failed to create history_visits table");
        
        (db_path, conn)
    }
    
    // Helper to insert mock history data into the database
    fn insert_mock_data(conn: &Connection) {
        // Insert URLs
        conn.execute(
            "INSERT INTO history_items (id, url, title, domain, visit_count, visit_time, last_visited_time) VALUES
            (1, 'https://example.com', 'Example Site', 'example.com', 3, 662688000, 662774400),
            (2, 'https://test.org/page1', 'Test Page 1', 'test.org', 1, 662860800, 662860800),
            (3, 'https://test.org/page2', 'Test Page 2', 'test.org', 2, 662947200, 663033600)",
            [],
        ).expect("Failed to insert mock URLs");
        
        // Insert visits
        conn.execute(
            "INSERT INTO history_visits (id, history_item, visit_time) VALUES
            (1, 1, 662688000),
            (2, 1, 662731200),
            (3, 1, 662774400),
            (4, 2, 662860800),
            (5, 3, 662947200),
            (6, 3, 663033600)",
            [],
        ).expect("Failed to insert mock visits");
    }

    #[test]
    fn test_mac_to_utc_conversion() {
        // Test the conversion function using a known macOS timestamp
        // macOS epoch is Jan 1, 2001, Unix epoch is Jan 1, 1970
        // Difference is 978307200 seconds
        
        // 1000000000 in macOS time should be 1978307200 in Unix time
        let mac_timestamp = 1000000000;
        let utc_time = mac_to_utc(mac_timestamp).expect("Timestamp conversion failed");
        
        // Expected result: 1978307200 seconds since Unix epoch
        let expected_unix_timestamp = mac_timestamp + MAC_TO_UNIX_EPOCH_OFFSET;
        assert_eq!(utc_time.timestamp(), expected_unix_timestamp);
    }
    
    #[test]
    fn test_extract_domain() {
        // Test the domain extraction function with various URLs
        let test_cases = [
            ("https://www.example.com", "www.example.com"),
            ("http://example.org", "example.org"),
            ("https://sub.domain.net/path?query=value", "sub.domain.net"),
            ("https://192.168.1.1:8080", "192.168.1.1"),
        ];
        
        for (url, expected) in test_cases {
            let result = extract_domain(url).expect("Domain extraction failed");
            assert_eq!(result, expected);
        }
        
        // Test invalid URL
        let result = extract_domain("not-a-valid-url");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_verify_safari_schema() {
        // Test schema verification with valid mock database
        let (db_path, conn) = create_mock_safari_db();
        
        let result = verify_safari_schema(&conn);
        assert!(result.is_ok());
        
        // Test with invalid database (create a new empty DB)
        let dir = tempdir().expect("Failed to create temp directory");
        let invalid_db_path = dir.path().join("invalid.db");
        let invalid_conn = Connection::open(&invalid_db_path).expect("Failed to create invalid db");
        
        let result = verify_safari_schema(&invalid_conn);
        assert!(result.is_err());
        
        match result {
            Err(ExtractionError::UnsupportedSchema(_)) => (), // Expected error type
            Err(e) => panic!("Unexpected error type: {:?}", e),
            Ok(_) => panic!("Expected error but got Ok"),
        }
    }
    
    #[test]
    fn test_extract_history() {
        // Create a mock database with test data
        let (db_path, conn) = create_mock_safari_db();
        insert_mock_data(&conn);
        
        // Close the connection to release the file
        drop(conn);
        
        // Test extraction
        let device_name = Some("Test Device".to_string());
        let result = extract_history(&db_path, device_name.clone());
        
        assert!(result.is_ok());
        let history_data = result.unwrap();
        
        // Verify extracted data
        assert_eq!(history_data.urls.len(), 3);
        assert_eq!(history_data.visits.len(), 6);
        assert_eq!(history_data.source.device_name, device_name);
        
        // Check URL properties
        let example_url = history_data.urls.iter()
            .find(|u| u.url == "https://example.com")
            .expect("Failed to find example.com URL");
            
        assert_eq!(example_url.title, Some("Example Site".to_string()));
        assert_eq!(example_url.domain, "example.com");
        
        // Check that visits point to valid URLs
        for visit in &history_data.visits {
            let url = history_data.urls.iter()
                .find(|u| u.id == visit.url_id)
                .expect("Visit references unknown URL ID");
            
            assert!(!url.url.is_empty());
        }
    }
    
    #[test]
    fn test_parse_history_db_multiple_files() {
        // Test handling of multiple files, including an invalid one
        
        // Create first valid mock database
        let (db_path1, conn1) = create_mock_safari_db();
        insert_mock_data(&conn1);
        drop(conn1);
        
        // Create second valid mock database
        let (db_path2, conn2) = create_mock_safari_db();
        insert_mock_data(&conn2);
        drop(conn2);
        
        // Create an invalid file
        let dir = tempdir().expect("Failed to create temp directory");
        let invalid_path = dir.path().join("not-a-db.txt");
        let mut file = File::create(&invalid_path).expect("Failed to create invalid file");
        writeln!(file, "This is not a SQLite database").expect("Failed to write to file");
        drop(file);
        
        // Test parsing of multiple files
        let files = vec![db_path1, invalid_path, db_path2];
        let device_names = vec![
            "Device 1".to_string(),
            "Invalid Device".to_string(),
            "Device 2".to_string(),
        ];
        
        let (successful, failed) = parse_history_db(&files, Some(&device_names));
        
        // We should have 2 successful extractions and 1 failure
        assert_eq!(successful.len(), 2);
        assert_eq!(failed.len(), 1);
        
        // Verify device names were assigned correctly
        assert_eq!(successful[0].source.device_name, Some("Device 1".to_string()));
        assert_eq!(successful[1].source.device_name, Some("Device 2".to_string()));
        
        // Verify failed file info
        assert_eq!(failed[0].path, files[1]);
    }
}
