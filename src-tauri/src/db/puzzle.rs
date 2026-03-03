use rusqlite::params;

use super::OptionalRow;
use crate::db::connection::Database;
use crate::error::DatabaseError;
use crate::models::puzzle::{
    Puzzle, PuzzleAttempt, PuzzleCategory, PuzzleFilter, PuzzleSessionStats,
};

fn row_to_puzzle(row: &rusqlite::Row) -> Result<Puzzle, rusqlite::Error> {
    Ok(Puzzle {
        id: row.get(0)?,
        fen: row.get(1)?,
        solution_moves: row.get(2)?,
        themes: row.get(3)?,
        category: PuzzleCategory::from_str(&row.get::<_, String>(4)?),
        difficulty: row.get::<_, i64>(5)? as u32,
        source_id: row.get(6)?,
        hints_json: row.get(7)?,
        explanation: row.get(8)?,
    })
}

impl Database {
    /// Get the next puzzle due for SRS review (earliest next_review in the past).
    pub fn get_next_due_puzzle(
        &self,
        player_id: &str,
        filter: &PuzzleFilter,
    ) -> Result<Option<Puzzle>, DatabaseError> {
        let category = filter
            .category
            .as_ref()
            .map(|c| c.as_str().to_string())
            .unwrap_or_else(|| "tactical".to_string());
        let min_diff = filter.min_difficulty.unwrap_or(0) as i64;
        let max_diff = filter.max_difficulty.unwrap_or(9999) as i64;

        let mut stmt = self.conn().prepare(
            "SELECT p.id, p.fen, p.solution_moves, p.themes, p.category,
                    p.difficulty, p.source_id, p.hints_json, p.explanation
             FROM puzzle p
             INNER JOIN puzzle_attempt pa ON pa.puzzle_id = p.id AND pa.player_id = ?1
             WHERE p.category = ?2
               AND p.difficulty >= ?3
               AND p.difficulty <= ?4
               AND pa.srs_next_review <= datetime('now')
             ORDER BY pa.srs_next_review ASC
             LIMIT 1",
        )?;

        let result = stmt
            .query_row(params![player_id, category, min_diff, max_diff], row_to_puzzle)
            .optional()?;

        Ok(result)
    }

    /// Get the next unseen puzzle near the target difficulty.
    pub fn get_next_new_puzzle(
        &self,
        player_id: &str,
        filter: &PuzzleFilter,
    ) -> Result<Option<Puzzle>, DatabaseError> {
        let category = filter
            .category
            .as_ref()
            .map(|c| c.as_str().to_string())
            .unwrap_or_else(|| "tactical".to_string());
        let min_diff = filter.min_difficulty.unwrap_or(0) as i64;
        let max_diff = filter.max_difficulty.unwrap_or(9999) as i64;
        let target = ((min_diff + max_diff) / 2) as f64;

        let mut stmt = self.conn().prepare(
            "SELECT p.id, p.fen, p.solution_moves, p.themes, p.category,
                    p.difficulty, p.source_id, p.hints_json, p.explanation
             FROM puzzle p
             WHERE p.category = ?1
               AND p.difficulty >= ?2
               AND p.difficulty <= ?3
               AND p.id NOT IN (
                   SELECT puzzle_id FROM puzzle_attempt WHERE player_id = ?4
               )
             ORDER BY ABS(p.difficulty - ?5)
             LIMIT 1",
        )?;

        let result = stmt
            .query_row(
                params![category, min_diff, max_diff, player_id, target],
                row_to_puzzle,
            )
            .optional()?;

        Ok(result)
    }

    /// Save a puzzle attempt with SRS data.
    pub fn save_puzzle_attempt(&self, attempt: &PuzzleAttempt) -> Result<(), DatabaseError> {
        self.conn().execute(
            "INSERT INTO puzzle_attempt (id, player_id, puzzle_id, solved, time_ms, hints_used,
                                         attempted_at, srs_interval, srs_ease, srs_next_review)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                attempt.id,
                attempt.player_id,
                attempt.puzzle_id,
                attempt.solved as i32,
                attempt.time_ms as i64,
                attempt.hints_used as i32,
                attempt.attempted_at,
                attempt.srs_interval,
                attempt.srs_ease,
                attempt.srs_next_review,
            ],
        )?;
        Ok(())
    }

    /// Get aggregate puzzle stats for a player.
    pub fn get_puzzle_stats(&self, player_id: &str) -> Result<PuzzleSessionStats, DatabaseError> {
        let mut stmt = self.conn().prepare(
            "SELECT COUNT(*), SUM(CASE WHEN solved = 1 THEN 1 ELSE 0 END),
                    COALESCE(AVG(time_ms), 0)
             FROM puzzle_attempt WHERE player_id = ?1",
        )?;

        let (total_attempts, total_solved, avg_time): (i64, i64, f64) =
            stmt.query_row(params![player_id], |row| {
                Ok((row.get(0)?, row.get::<_, Option<i64>>(1)?.unwrap_or(0), row.get(2)?))
            })?;

        // Compute current streak (consecutive solved, most recent first)
        let mut streak_stmt = self.conn().prepare(
            "SELECT solved FROM puzzle_attempt
             WHERE player_id = ?1
             ORDER BY attempted_at DESC",
        )?;
        let solved_iter = streak_stmt.query_map(params![player_id], |row| {
            row.get::<_, bool>(0)
        })?;

        let mut current_streak: u32 = 0;
        for solved in solved_iter {
            if solved? {
                current_streak += 1;
            } else {
                break;
            }
        }

        // Best streak (scan all attempts chronologically)
        let mut best_streak_stmt = self.conn().prepare(
            "SELECT solved FROM puzzle_attempt
             WHERE player_id = ?1
             ORDER BY attempted_at ASC",
        )?;
        let all_solved = best_streak_stmt.query_map(params![player_id], |row| {
            row.get::<_, bool>(0)
        })?;

        let mut best_streak: u32 = 0;
        let mut running: u32 = 0;
        for solved in all_solved {
            if solved? {
                running += 1;
                best_streak = best_streak.max(running);
            } else {
                running = 0;
            }
        }
        best_streak = best_streak.max(current_streak);

        Ok(PuzzleSessionStats {
            total_attempts: total_attempts as u32,
            total_solved: total_solved as u32,
            average_time_ms: avg_time as u64,
            current_streak,
            best_streak,
        })
    }

    /// Bulk import puzzles (INSERT OR IGNORE for dedup by ID).
    pub fn import_puzzles(&self, puzzles: &[Puzzle]) -> Result<usize, DatabaseError> {
        let tx = self.conn().unchecked_transaction()?;
        let mut count = 0;

        {
            let mut stmt = tx.prepare(
                "INSERT OR IGNORE INTO puzzle (id, fen, solution_moves, themes, category,
                                                difficulty, source_id, hints_json, explanation)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )?;

            for p in puzzles {
                let inserted = stmt.execute(params![
                    p.id,
                    p.fen,
                    p.solution_moves,
                    p.themes,
                    p.category.as_str(),
                    p.difficulty as i64,
                    p.source_id,
                    p.hints_json,
                    p.explanation,
                ])?;
                count += inserted;
            }
        }

        tx.commit()?;
        Ok(count)
    }

    /// Get the count of puzzles currently in the database.
    pub fn get_puzzle_count(&self) -> Result<u32, DatabaseError> {
        let count: i64 = self
            .conn()
            .query_row("SELECT COUNT(*) FROM puzzle", [], |row| row.get(0))?;
        Ok(count as u32)
    }

    /// Get the previous attempt count for a player on a specific puzzle.
    pub fn get_attempt_count(
        &self,
        player_id: &str,
        puzzle_id: &str,
    ) -> Result<u32, DatabaseError> {
        let count: i64 = self.conn().query_row(
            "SELECT COUNT(*) FROM puzzle_attempt WHERE player_id = ?1 AND puzzle_id = ?2",
            params![player_id, puzzle_id],
            |row| row.get(0),
        )?;
        Ok(count as u32)
    }

    /// Get the latest SRS state for a player/puzzle pair.
    pub fn get_latest_srs(
        &self,
        player_id: &str,
        puzzle_id: &str,
    ) -> Result<Option<(f64, f64)>, DatabaseError> {
        let result = self
            .conn()
            .query_row(
                "SELECT srs_interval, srs_ease FROM puzzle_attempt
                 WHERE player_id = ?1 AND puzzle_id = ?2
                 ORDER BY attempted_at DESC LIMIT 1",
                params![player_id, puzzle_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        Ok(result)
    }

    /// Get solve rate for the most recent N puzzle attempts.
    /// Returns (solve_rate, count) where count is the actual number of recent attempts found.
    pub fn get_recent_puzzle_solve_rate(
        &self,
        player_id: &str,
        limit: u32,
    ) -> Result<(f64, u32), DatabaseError> {
        let mut stmt = self.conn().prepare(
            "SELECT solved FROM puzzle_attempt WHERE player_id = ?1 ORDER BY attempted_at DESC LIMIT ?2",
        )?;
        let results: Vec<bool> = stmt
            .query_map(params![player_id, limit], |row| row.get::<_, bool>(0))?
            .collect::<Result<Vec<_>, _>>()?;

        let count = results.len() as u32;
        if count == 0 {
            return Ok((0.0, 0));
        }
        let solved = results.iter().filter(|&&s| s).count() as f64;
        Ok((solved / count as f64, count))
    }

    /// Get hint usage rate for the most recent N puzzle attempts.
    pub fn get_puzzle_hint_usage_rate(
        &self,
        player_id: &str,
        limit: u32,
    ) -> Result<f64, DatabaseError> {
        let mut stmt = self.conn().prepare(
            "SELECT hints_used FROM puzzle_attempt WHERE player_id = ?1 ORDER BY attempted_at DESC LIMIT ?2",
        )?;
        let results: Vec<i32> = stmt
            .query_map(params![player_id, limit], |row| row.get::<_, i32>(0))?
            .collect::<Result<Vec<_>, _>>()?;

        if results.is_empty() {
            return Ok(0.0);
        }
        let used = results.iter().filter(|&&h| h > 0).count() as f64;
        Ok(used / results.len() as f64)
    }

    /// Get all unique themes from puzzles in the database.
    pub fn get_puzzle_themes(&self) -> Result<Vec<String>, DatabaseError> {
        let mut stmt = self
            .conn()
            .prepare("SELECT DISTINCT themes FROM puzzle WHERE themes != ''")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        let mut theme_set = std::collections::BTreeSet::new();
        for row in rows {
            let themes_str = row?;
            for theme in themes_str.split(',') {
                let t = theme.trim();
                if !t.is_empty() {
                    theme_set.insert(t.to_string());
                }
            }
        }

        Ok(theme_set.into_iter().collect())
    }
}
