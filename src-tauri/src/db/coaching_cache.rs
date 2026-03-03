use crate::error::DatabaseError;

use super::connection::Database;

impl Database {
    /// Look up a cached coaching text by cache key.
    /// Returns `(coaching_text, player_level)` if found and not expired.
    pub fn get_cached_coaching(
        &self,
        cache_key: &str,
    ) -> Result<Option<(String, String)>, DatabaseError> {
        let result = self.conn().query_row(
            "SELECT coaching_text, player_level FROM coaching_cache \
             WHERE cache_key = ?1 AND expires_at > datetime('now')",
            [cache_key],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        );

        match result {
            Ok(pair) => Ok(Some(pair)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DatabaseError::Sqlite(e)),
        }
    }

    /// Store a coaching text in the cache.
    #[allow(dead_code)]
    pub fn set_cached_coaching(
        &self,
        cache_key: &str,
        text: &str,
        level: &str,
        classification: &str,
        fen: &str,
        expires_days: u32,
    ) -> Result<(), DatabaseError> {
        self.conn().execute(
            "INSERT OR REPLACE INTO coaching_cache \
             (cache_key, coaching_text, player_level, classification, fen, expires_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now', '+' || ?6 || ' days'))",
            rusqlite::params![cache_key, text, level, classification, fen, expires_days],
        )?;
        Ok(())
    }

    /// Remove expired cache entries.
    #[allow(dead_code)]
    pub fn cleanup_expired_cache(&self) -> Result<u64, DatabaseError> {
        let deleted = self.conn().execute(
            "DELETE FROM coaching_cache WHERE expires_at < datetime('now')",
            [],
        )?;
        Ok(deleted as u64)
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    fn setup_test_db() -> super::Database {
        // Create an in-memory database with the required schema
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE coaching_cache (
                cache_key     TEXT PRIMARY KEY,
                coaching_text TEXT NOT NULL,
                player_level  TEXT NOT NULL,
                classification TEXT,
                fen           TEXT,
                created_at    TEXT NOT NULL DEFAULT (datetime('now')),
                expires_at    TEXT NOT NULL
            );
            CREATE INDEX idx_coaching_cache_expires ON coaching_cache(expires_at);",
        )
        .unwrap();

        // Use a test-only constructor. We access conn via the public interface.
        // Since Database wraps Connection, we construct it directly for tests.
        super::Database::from_connection(conn)
    }

    #[test]
    fn insert_and_retrieve_cache() {
        let db = setup_test_db();
        db.set_cached_coaching("key1", "Great move!", "beginner", "best", "fen1", 30)
            .unwrap();

        let result = db.get_cached_coaching("key1").unwrap();
        assert!(result.is_some());
        let (text, level) = result.unwrap();
        assert_eq!(text, "Great move!");
        assert_eq!(level, "beginner");
    }

    #[test]
    fn missing_key_returns_none() {
        let db = setup_test_db();
        let result = db.get_cached_coaching("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn expired_entry_returns_none() {
        let db = setup_test_db();
        // Insert with expiry in the past
        db.conn()
            .execute(
                "INSERT INTO coaching_cache \
                 (cache_key, coaching_text, player_level, classification, fen, expires_at) \
                 VALUES ('expired', 'old', 'beginner', 'blunder', 'fen', datetime('now', '-1 day'))",
                [],
            )
            .unwrap();

        let result = db.get_cached_coaching("expired").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn cleanup_removes_expired() {
        let db = setup_test_db();
        // Insert an expired entry
        db.conn()
            .execute(
                "INSERT INTO coaching_cache \
                 (cache_key, coaching_text, player_level, classification, fen, expires_at) \
                 VALUES ('old', 'text', 'beginner', 'blunder', 'fen', datetime('now', '-1 day'))",
                [],
            )
            .unwrap();
        // Insert a valid entry
        db.set_cached_coaching("new", "fresh", "intermediate", "best", "fen2", 30)
            .unwrap();

        let deleted = db.cleanup_expired_cache().unwrap();
        assert_eq!(deleted, 1);

        // Valid entry still exists
        assert!(db.get_cached_coaching("new").unwrap().is_some());
    }

    #[test]
    fn upsert_replaces_existing() {
        let db = setup_test_db();
        db.set_cached_coaching("key1", "First", "beginner", "best", "fen1", 30)
            .unwrap();
        db.set_cached_coaching("key1", "Second", "intermediate", "best", "fen1", 30)
            .unwrap();

        let (text, level) = db.get_cached_coaching("key1").unwrap().unwrap();
        assert_eq!(text, "Second");
        assert_eq!(level, "intermediate");
    }
}
