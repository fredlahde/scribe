//! History module for storing and retrieving transcription records.

use std::path::PathBuf;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Maximum number of transcriptions to keep in history
const MAX_HISTORY_SIZE: i64 = 50;

/// Sample rate of Whisper audio (16kHz)
const WHISPER_SAMPLE_RATE: f64 = 16000.0;

/// A single transcription record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcription {
    pub id: i64,
    pub text: String,
    pub language: String,
    pub duration_ms: i64,
    pub word_count: i32,
    pub created_at: String,
}

/// Thread-safe wrapper around the database connection
pub struct HistoryDb {
    conn: Mutex<Connection>,
}

impl HistoryDb {
    /// Create a new database connection, initializing the schema if needed
    pub fn new(app_data_dir: PathBuf) -> Result<Self> {
        // Ensure the directory exists
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| Error::Database(format!("failed to create app data dir: {}", e)))?;

        let db_path = app_data_dir.join("history.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| Error::Database(format!("failed to open database: {}", e)))?;

        // Initialize schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL,
                language TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                word_count INTEGER NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| Error::Database(format!("failed to create table: {}", e)))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Save a new transcription to the database
    pub fn save_transcription(
        &self,
        text: &str,
        language: &str,
        sample_count: usize,
    ) -> Result<Transcription> {
        let mut conn = self.conn.lock().unwrap();

        // Calculate duration from sample count (16kHz sample rate)
        let duration_ms = ((sample_count as f64 / WHISPER_SAMPLE_RATE) * 1000.0) as i64;

        // Count words using unicode-aware splitting
        let word_count = text
            .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .filter(|s| !s.is_empty())
            .count() as i32;

        // Current timestamp in ISO 8601 format
        let created_at: DateTime<Utc> = Utc::now();
        let created_at_str = created_at.to_rfc3339();

        // Wrap both INSERT and cleanup DELETE in a single transaction
        let tx = conn
            .transaction()
            .map_err(|e| Error::Database(format!("failed to start transaction: {}", e)))?;

        tx.execute(
            "INSERT INTO transcriptions (text, language, duration_ms, word_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![text, language, duration_ms, word_count, created_at_str],
        )
        .map_err(|e| Error::Database(format!("failed to insert transcription: {}", e)))?;

        let id = tx.last_insert_rowid();

        // Enforce max history size by deleting oldest entries (within same transaction)
        tx.execute(
            "DELETE FROM transcriptions WHERE id NOT IN (
                SELECT id FROM transcriptions ORDER BY created_at DESC LIMIT ?1
            )",
            params![MAX_HISTORY_SIZE],
        )
        .map_err(|e| Error::Database(format!("failed to cleanup old entries: {}", e)))?;

        tx.commit()
            .map_err(|e| Error::Database(format!("failed to commit transaction: {}", e)))?;

        Ok(Transcription {
            id,
            text: text.to_string(),
            language: language.to_string(),
            duration_ms,
            word_count,
            created_at: created_at_str,
        })
    }

    /// Get the most recent transcriptions
    pub fn get_history(&self, limit: i64) -> Result<Vec<Transcription>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT id, text, language, duration_ms, word_count, created_at
                 FROM transcriptions
                 ORDER BY created_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| Error::Database(format!("failed to prepare query: {}", e)))?;

        let transcriptions = stmt
            .query_map([limit], |row| {
                Ok(Transcription {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    language: row.get(2)?,
                    duration_ms: row.get(3)?,
                    word_count: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| Error::Database(format!("failed to query history: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Database(format!("failed to collect results: {}", e)))?;

        Ok(transcriptions)
    }

    /// Delete a transcription by ID
    pub fn delete_transcription(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let rows_affected = conn
            .execute("DELETE FROM transcriptions WHERE id = ?1", params![id])
            .map_err(|e| Error::Database(format!("failed to delete transcription: {}", e)))?;

        Ok(rows_affected > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_get_transcription() {
        let temp_dir = tempdir().unwrap();
        let db = HistoryDb::new(temp_dir.path().to_path_buf()).unwrap();

        // Save a transcription (16000 samples = 1 second at 16kHz)
        let transcription = db.save_transcription("Hello world", "en", 16000).unwrap();

        assert_eq!(transcription.text, "Hello world");
        assert_eq!(transcription.language, "en");
        assert_eq!(transcription.duration_ms, 1000);
        assert_eq!(transcription.word_count, 2);

        // Get history
        let history = db.get_history(10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].text, "Hello world");
    }

    #[test]
    fn test_delete_transcription() {
        let temp_dir = tempdir().unwrap();
        let db = HistoryDb::new(temp_dir.path().to_path_buf()).unwrap();

        let transcription = db.save_transcription("Test", "de", 8000).unwrap();
        let deleted = db.delete_transcription(transcription.id).unwrap();
        assert!(deleted);

        let history = db.get_history(10).unwrap();
        assert!(history.is_empty());
    }

    #[test]
    fn test_max_history_cleanup() {
        let temp_dir = tempdir().unwrap();
        let db = HistoryDb::new(temp_dir.path().to_path_buf()).unwrap();

        // Insert more than MAX_HISTORY_SIZE entries
        for i in 0..55 {
            db.save_transcription(&format!("Entry {}", i), "en", 16000)
                .unwrap();
        }

        // Should only have MAX_HISTORY_SIZE entries
        let history = db.get_history(100).unwrap();
        assert_eq!(history.len(), MAX_HISTORY_SIZE as usize);
    }
}
