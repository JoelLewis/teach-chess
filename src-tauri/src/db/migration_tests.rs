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
        let db = Database::open_in_memory().expect("first migration run should succeed");

        // Running migrations again should not error
        db.run_migrations()
            .expect("migrations should be safe to re-run (idempotent)");
    }

    #[test]
    fn dashboard_queries_on_fresh_db() {
        let db = Database::open_in_memory().expect("migrations should apply");

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
        let games = db
            .get_game_history(5, 0)
            .expect("get_game_history should work on fresh db");
        assert!(games.is_empty(), "no games yet");

        // get_puzzle_stats
        let stats = db
            .get_puzzle_stats(player_id)
            .expect("get_puzzle_stats should work on fresh db");
        assert_eq!(stats.total_attempts, 0);

        // get_skill_profile
        let profile = db
            .get_skill_profile(player_id)
            .expect("get_skill_profile should work on fresh db");
        assert!(profile.ratings.is_empty(), "no ratings yet");

        // get_today_counts
        let (games_today, puzzles_today) = db
            .get_today_counts(player_id, "2026-01-01")
            .expect("get_today_counts should work on fresh db");
        assert_eq!(games_today, 0);
        assert_eq!(puzzles_today, 0);

        // get_activity_dates
        let dates = db
            .get_activity_dates(player_id)
            .expect("get_activity_dates should work on fresh db");
        assert!(dates.is_empty(), "no activity dates yet");
    }

    #[test]
    fn sm2_state_converts_to_fsrs_on_migration() {
        use crate::db::srs::SrsItemKind;
        use rs_fsrs::State;

        // Build a pre-008 database: migrations 001–007 plus SM-2 rows the
        // way the old code wrote them.
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        for sql in [
            include_str!("../../migrations/001_initial.sql"),
            include_str!("../../migrations/002_coaching_text.sql"),
            include_str!("../../migrations/003_coaching_cache.sql"),
            include_str!("../../migrations/004_problems.sql"),
            include_str!("../../migrations/005_repertoire.sql"),
            include_str!("../../migrations/006_assessment.sql"),
            include_str!("../../migrations/007_opponent_personality.sql"),
        ] {
            // Mirror the runner: 002/007 add columns that 001 may already have
            if let Err(e) = conn.execute_batch(sql) {
                assert!(e.to_string().contains("duplicate column"), "{e}");
            }
        }

        conn.execute_batch(
            "INSERT INTO player (id, display_name) VALUES ('p1', 'Test');
             INSERT INTO puzzle (id, fen, solution_moves) VALUES ('pz1', 'fen1', 'e2e4');
             INSERT INTO opening (id, name, color, moves)
                VALUES ('italian', 'Italian', 'white', 'e2e4');
             INSERT INTO repertoire_entry (id, player_id, opening_id, position_fen, move_uci)
                VALUES ('re1', 'p1', 'italian', 'startfen', 'e2e4');
             -- SM-2 history: one lapse, then a pass at interval 6 / ease 2.28
             INSERT INTO puzzle_attempt (id, player_id, puzzle_id, solved, time_ms, hints_used,
                                         attempted_at, srs_interval, srs_ease, srs_next_review)
                VALUES ('a1', 'p1', 'pz1', 0, 5000, 0,
                        '2026-01-01 10:00:00', 1.0, 2.18, '2026-01-02 10:00:00');
             INSERT INTO puzzle_attempt (id, player_id, puzzle_id, solved, time_ms, hints_used,
                                         attempted_at, srs_interval, srs_ease, srs_next_review)
                VALUES ('a2', 'p1', 'pz1', 1, 5000, 0,
                        '2026-01-05 10:00:00', 6.0, 2.28, '2026-01-11 10:00:00');
             INSERT INTO repertoire_drill_attempt (id, player_id, repertoire_entry_id, correct,
                                                   time_ms, attempted_at, srs_interval, srs_ease,
                                                   srs_next_review)
                VALUES ('d1', 'p1', 're1', 1, 3000,
                        '2026-01-28 00:00:00', 4.0, 2.5, '2026-02-01 00:00:00');",
        )
        .unwrap();

        // Run the full migration chain (008 converts SM-2 → FSRS)
        let db = Database::from_connection(conn);
        db.run_migrations().unwrap();

        // Puzzle card: latest attempt wins, due date is preserved
        let card = db.get_srs_card("p1", SrsItemKind::Puzzle, "pz1").unwrap();
        assert_eq!(
            card.due.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2026-01-11 10:00:00",
            "due date must carry over so the review queue does not reset"
        );
        assert!(
            (card.stability - 6.0).abs() < 1e-9,
            "stability seeds from interval"
        );
        // difficulty = clamp(5 + (2.5 - 2.28) * (5 / 1.2), 1, 10)
        assert!((card.difficulty - (5.0 + 0.22 * (5.0 / 1.2))).abs() < 1e-6);
        assert_eq!(card.reps, 2);
        assert_eq!(card.lapses, 1);
        assert_eq!(card.state, State::Review);
        assert_eq!(card.scheduled_days, 6);
        // last_review reconstructed as due - interval
        assert_eq!(
            card.last_review.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2026-01-05 10:00:00"
        );

        // Drill card: default ease 2.5 maps to mid difficulty 5.0
        let drill = db.get_srs_card("p1", SrsItemKind::Drill, "re1").unwrap();
        assert_eq!(
            drill.due.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2026-02-01 00:00:00"
        );
        assert!((drill.stability - 4.0).abs() < 1e-9);
        assert!((drill.difficulty - 5.0).abs() < 1e-6);
        assert_eq!(drill.reps, 1);
        assert_eq!(drill.lapses, 0);

        // SM-2 columns are gone from the attempt tables
        for table in ["puzzle_attempt", "repertoire_drill_attempt"] {
            let result = db
                .conn()
                .prepare(&format!("SELECT srs_interval FROM {table}"));
            assert!(result.is_err(), "{table}.srs_interval should be dropped");
        }

        // Re-running migrations is still safe post-conversion
        db.run_migrations()
            .expect("008 must be guarded and re-runnable");
    }

    #[test]
    fn all_migration_files_are_registered() {
        // This is the single highest-value test: it catches "file exists on disk
        // but the runner forgot to register it" at test time.
        let migrations_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");

        let connection_rs =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/db/connection.rs"))
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
