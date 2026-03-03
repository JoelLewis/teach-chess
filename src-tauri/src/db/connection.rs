use std::fs;

use rusqlite::Connection;
use tauri::Manager;
use tracing::info;

use crate::error::DatabaseError;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(app: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let app_data_dir = app.path().app_data_dir()?;
        fs::create_dir_all(&app_data_dir)?;

        let db_path = app_data_dir.join("chess-mentor.db");
        info!("Opening database at {}", db_path.display());

        let conn = Connection::open(&db_path)?;

        // Enable WAL mode for better concurrent read performance
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let db = Self { conn };
        db.run_migrations()?;

        Ok(db)
    }

    fn run_migrations(&self) -> Result<(), DatabaseError> {
        let migration_sql = include_str!("../../migrations/001_initial.sql");
        self.conn
            .execute_batch(migration_sql)
            .map_err(DatabaseError::Sqlite)?;

        // 002: Add coaching_text column (safe to re-run — ignore "duplicate column" error)
        let m002 = include_str!("../../migrations/002_coaching_text.sql");
        if let Err(e) = self.conn.execute_batch(m002) {
            let msg = e.to_string();
            if !msg.contains("duplicate column") {
                return Err(DatabaseError::Sqlite(e));
            }
        }

        // 003: Coaching cache table (IF NOT EXISTS — safe to re-run)
        let m003 = include_str!("../../migrations/003_coaching_cache.sql");
        self.conn
            .execute_batch(m003)
            .map_err(DatabaseError::Sqlite)?;

        // 004: Puzzle tables (IF NOT EXISTS — safe to re-run)
        let m004 = include_str!("../../migrations/004_problems.sql");
        self.conn
            .execute_batch(m004)
            .map_err(DatabaseError::Sqlite)?;

        // 005: Repertoire tables (IF NOT EXISTS — safe to re-run)
        let m005 = include_str!("../../migrations/005_repertoire.sql");
        self.conn
            .execute_batch(m005)
            .map_err(DatabaseError::Sqlite)?;

        // 006: Skill rating table (IF NOT EXISTS — safe to re-run)
        let m006 = include_str!("../../migrations/006_assessment.sql");
        self.conn
            .execute_batch(m006)
            .map_err(DatabaseError::Sqlite)?;

        info!("Database migrations applied");
        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Create a Database from an existing connection (for testing).
    #[cfg(test)]
    pub fn from_connection(conn: Connection) -> Self {
        Self { conn }
    }
}
