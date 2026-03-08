#[cfg(test)]
mod tests {
    use crate::db::connection::Database;
    use std::fs;
    use std::path::Path;

    #[test]
    fn all_migrations_apply_cleanly() {
        // This would have caught the missing 007 registration bug.
        // If any migration fails, open_in_memory() returns an error.
        let db = Database::open_in_memory()
            .expect("all migrations should apply cleanly on a fresh database");

        // Verify we can at least query the schema
        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get(0),
            )
            .expect("should be able to query schema");

        assert!(count > 0, "should have created at least one table");
    }

    #[test]
    fn migrations_are_idempotent() {
        let db = Database::open_in_memory()
            .expect("first migration run should succeed");

        // Running migrations again should not error
        db.run_migrations()
            .expect("migrations should be safe to re-run (idempotent)");
    }

    #[test]
    fn dashboard_queries_on_fresh_db() {
        let db = Database::open_in_memory()
            .expect("migrations should apply");

        // Insert a test player
        db.conn()
            .execute(
                "INSERT INTO player (id, display_name) VALUES (?1, ?2)",
                rusqlite::params!["test-player-1", "TestPlayer"],
            )
            .expect("should insert test player");

        let player_id = "test-player-1";

        // These are the same queries the dashboard command uses:

        // get_game_history
        let games = db.get_game_history(5, 0)
            .expect("get_game_history should work on fresh db");
        assert!(games.is_empty(), "no games yet");

        // get_puzzle_stats
        let stats = db.get_puzzle_stats(player_id)
            .expect("get_puzzle_stats should work on fresh db");
        assert_eq!(stats.total_attempts, 0);

        // get_skill_profile
        let profile = db.get_skill_profile(player_id)
            .expect("get_skill_profile should work on fresh db");
        assert!(profile.ratings.is_empty(), "no ratings yet");

        // get_today_counts
        let (games_today, puzzles_today) = db.get_today_counts(player_id, "2026-01-01")
            .expect("get_today_counts should work on fresh db");
        assert_eq!(games_today, 0);
        assert_eq!(puzzles_today, 0);

        // get_activity_dates
        let dates = db.get_activity_dates(player_id)
            .expect("get_activity_dates should work on fresh db");
        assert!(dates.is_empty(), "no activity dates yet");
    }

    #[test]
    fn all_migration_files_are_registered() {
        // This is the single highest-value test: it catches "file exists on disk
        // but the runner forgot to register it" at test time.
        let migrations_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");

        let connection_rs = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/db/connection.rs"),
        )
        .expect("should be able to read connection.rs");

        let mut sql_files: Vec<String> = fs::read_dir(&migrations_dir)
            .expect("migrations directory should exist")
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".sql") {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();

        sql_files.sort();

        assert!(
            !sql_files.is_empty(),
            "should find at least one migration file"
        );

        for file in &sql_files {
            let include_pattern = format!("migrations/{file}");
            assert!(
                connection_rs.contains(&include_pattern),
                "Migration file '{file}' exists on disk but is not referenced in connection.rs. \
                 Add `include_str!(\"../../migrations/{file}\")` to run_migrations()."
            );
        }
    }
}
