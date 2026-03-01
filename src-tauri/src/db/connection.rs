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
        info!("Database migrations applied");
        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
