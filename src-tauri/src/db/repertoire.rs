use rusqlite::params;
use shakmaty::{fen::Fen, san::San, uci::UciMove, Chess, EnPassantMode, Position};

use super::OptionalRow;
use crate::db::connection::Database;
use crate::error::DatabaseError;
use crate::models::repertoire::{
    DrillAttempt, DrillStats, Opening, OpeningPosition, RepertoireEntry, RepertoireFilter,
};

fn row_to_repertoire_entry(row: &rusqlite::Row) -> Result<RepertoireEntry, rusqlite::Error> {
    Ok(RepertoireEntry {
        id: row.get(0)?,
        player_id: row.get(1)?,
        opening_id: row.get(2)?,
        position_fen: row.get(3)?,
        move_uci: row.get(4)?,
        move_san: row.get(5)?,
        notes: row.get(6)?,
    })
}

impl Database {
    /// List openings with optional filter.
    pub fn get_openings(&self, filter: &RepertoireFilter) -> Result<Vec<Opening>, DatabaseError> {
        let mut sql = String::from(
            "SELECT id, name, eco, color, description, moves, themes, difficulty FROM opening WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut idx = 1;

        if let Some(ref color) = filter.color {
            sql.push_str(&format!(" AND color = ?{idx}"));
            param_values.push(Box::new(color.clone()));
            idx += 1;
        }
        if let Some(ref eco_prefix) = filter.eco_prefix {
            sql.push_str(&format!(" AND eco LIKE ?{idx}"));
            param_values.push(Box::new(format!("{eco_prefix}%")));
            idx += 1;
        }
        if let Some(min) = filter.min_difficulty {
            sql.push_str(&format!(" AND difficulty >= ?{idx}"));
            param_values.push(Box::new(min as i64));
            idx += 1;
        }
        if let Some(max) = filter.max_difficulty {
            sql.push_str(&format!(" AND difficulty <= ?{idx}"));
            param_values.push(Box::new(max as i64));
            let _ = idx + 1;
        }
        sql.push_str(" ORDER BY eco, name");

        let mut stmt = self.conn().prepare(&sql)?;
        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            Ok(Opening {
                id: row.get(0)?,
                name: row.get(1)?,
                eco: row.get(2)?,
                color: row.get(3)?,
                description: row.get(4)?,
                moves: row.get(5)?,
                themes: row.get(6)?,
                difficulty: row.get::<_, i64>(7)? as u32,
            })
        })?;

        let mut openings = Vec::new();
        for row in rows {
            openings.push(row?);
        }
        Ok(openings)
    }

    /// Get a single opening by ID.
    pub fn get_opening(&self, id: &str) -> Result<Option<Opening>, DatabaseError> {
        let result = self
            .conn()
            .query_row(
                "SELECT id, name, eco, color, description, moves, themes, difficulty
                 FROM opening WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Opening {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        eco: row.get(2)?,
                        color: row.get(3)?,
                        description: row.get(4)?,
                        moves: row.get(5)?,
                        themes: row.get(6)?,
                        difficulty: row.get::<_, i64>(7)? as u32,
                    })
                },
            )
            .optional()?;
        Ok(result)
    }

    /// Get all positions for an opening (ordered by move index).
    pub fn get_opening_positions(
        &self,
        opening_id: &str,
    ) -> Result<Vec<OpeningPosition>, DatabaseError> {
        let mut stmt = self.conn().prepare(
            "SELECT id, opening_id, fen, move_index, parent_fen, move_uci, move_san
             FROM opening_position WHERE opening_id = ?1 ORDER BY move_index",
        )?;
        let rows = stmt.query_map(params![opening_id], |row| {
            Ok(OpeningPosition {
                id: row.get(0)?,
                opening_id: row.get(1)?,
                fen: row.get(2)?,
                move_index: row.get::<_, i64>(3)? as u32,
                parent_fen: row.get(4)?,
                move_uci: row.get(5)?,
                move_san: row.get(6)?,
            })
        })?;

        let mut positions = Vec::new();
        for row in rows {
            positions.push(row?);
        }
        Ok(positions)
    }

    /// Get player's repertoire entries for an opening.
    pub fn get_repertoire_entries(
        &self,
        player_id: &str,
        opening_id: &str,
    ) -> Result<Vec<RepertoireEntry>, DatabaseError> {
        let mut stmt = self.conn().prepare(
            "SELECT id, player_id, opening_id, position_fen, move_uci, move_san, notes
             FROM repertoire_entry WHERE player_id = ?1 AND opening_id = ?2
             ORDER BY rowid",
        )?;
        let rows = stmt.query_map(params![player_id, opening_id], row_to_repertoire_entry)?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }

    /// Get all repertoire entries for a player (grouped by opening).
    #[allow(dead_code)]
    pub fn get_all_repertoire_entries(
        &self,
        player_id: &str,
    ) -> Result<Vec<RepertoireEntry>, DatabaseError> {
        let mut stmt = self.conn().prepare(
            "SELECT id, player_id, opening_id, position_fen, move_uci, move_san, notes
             FROM repertoire_entry WHERE player_id = ?1
             ORDER BY opening_id, rowid",
        )?;
        let rows = stmt.query_map(params![player_id], row_to_repertoire_entry)?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }

    /// Add a repertoire entry (upsert — replaces move at same position).
    pub fn add_repertoire_entry(&self, entry: &RepertoireEntry) -> Result<(), DatabaseError> {
        self.conn().execute(
            "INSERT INTO repertoire_entry (id, player_id, opening_id, position_fen, move_uci, move_san, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(player_id, opening_id, position_fen) DO UPDATE SET
               move_uci = excluded.move_uci,
               move_san = excluded.move_san,
               notes = excluded.notes",
            params![
                entry.id,
                entry.player_id,
                entry.opening_id,
                entry.position_fen,
                entry.move_uci,
                entry.move_san,
                entry.notes,
            ],
        )?;
        Ok(())
    }

    /// Remove a repertoire entry by ID.
    pub fn remove_repertoire_entry(&self, id: &str) -> Result<(), DatabaseError> {
        self.conn()
            .execute("DELETE FROM repertoire_entry WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Get next SRS-due repertoire entry for drilling.
    pub fn get_next_due_drill_entry(
        &self,
        player_id: &str,
        opening_id: &str,
    ) -> Result<Option<RepertoireEntry>, DatabaseError> {
        let result = self
            .conn()
            .query_row(
                "SELECT re.id, re.player_id, re.opening_id, re.position_fen, re.move_uci, re.move_san, re.notes
                 FROM repertoire_entry re
                 INNER JOIN repertoire_drill_attempt rda ON rda.repertoire_entry_id = re.id
                   AND rda.player_id = ?1
                 WHERE re.opening_id = ?2 AND re.player_id = ?1
                   AND rda.srs_next_review <= datetime('now')
                 ORDER BY rda.srs_next_review ASC
                 LIMIT 1",
                params![player_id, opening_id],
                |row| {
                    Ok(RepertoireEntry {
                        id: row.get(0)?,
                        player_id: row.get(1)?,
                        opening_id: row.get(2)?,
                        position_fen: row.get(3)?,
                        move_uci: row.get(4)?,
                        move_san: row.get(5)?,
                        notes: row.get(6)?,
                    })
                },
            )
            .optional()?;
        Ok(result)
    }

    /// Get a repertoire entry that has never been drilled.
    pub fn get_next_new_drill_entry(
        &self,
        player_id: &str,
        opening_id: &str,
    ) -> Result<Option<RepertoireEntry>, DatabaseError> {
        let result = self
            .conn()
            .query_row(
                "SELECT re.id, re.player_id, re.opening_id, re.position_fen, re.move_uci, re.move_san, re.notes
                 FROM repertoire_entry re
                 WHERE re.opening_id = ?1 AND re.player_id = ?2
                   AND re.id NOT IN (
                     SELECT repertoire_entry_id FROM repertoire_drill_attempt WHERE player_id = ?2
                   )
                 ORDER BY re.rowid
                 LIMIT 1",
                params![opening_id, player_id],
                |row| {
                    Ok(RepertoireEntry {
                        id: row.get(0)?,
                        player_id: row.get(1)?,
                        opening_id: row.get(2)?,
                        position_fen: row.get(3)?,
                        move_uci: row.get(4)?,
                        move_san: row.get(5)?,
                        notes: row.get(6)?,
                    })
                },
            )
            .optional()?;
        Ok(result)
    }

    /// Save a drill attempt with SRS data.
    pub fn save_drill_attempt(&self, attempt: &DrillAttempt) -> Result<(), DatabaseError> {
        self.conn().execute(
            "INSERT INTO repertoire_drill_attempt
             (id, player_id, repertoire_entry_id, correct, time_ms, attempted_at, srs_interval, srs_ease, srs_next_review)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                attempt.id,
                attempt.player_id,
                attempt.repertoire_entry_id,
                attempt.correct as i32,
                attempt.time_ms as i64,
                attempt.srs_next_review, // attempted_at defaults in DB, but we set it
                attempt.srs_interval,
                attempt.srs_ease,
                attempt.srs_next_review,
            ],
        )?;
        Ok(())
    }

    /// Get latest SRS state for a drill entry.
    pub fn get_latest_drill_srs(
        &self,
        player_id: &str,
        entry_id: &str,
    ) -> Result<Option<(f64, f64)>, DatabaseError> {
        let result = self
            .conn()
            .query_row(
                "SELECT srs_interval, srs_ease FROM repertoire_drill_attempt
                 WHERE player_id = ?1 AND repertoire_entry_id = ?2
                 ORDER BY attempted_at DESC LIMIT 1",
                params![player_id, entry_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        Ok(result)
    }

    /// Get drill attempt count for a specific entry.
    pub fn get_drill_attempt_count(
        &self,
        player_id: &str,
        entry_id: &str,
    ) -> Result<u32, DatabaseError> {
        let count: i64 = self.conn().query_row(
            "SELECT COUNT(*) FROM repertoire_drill_attempt
             WHERE player_id = ?1 AND repertoire_entry_id = ?2",
            params![player_id, entry_id],
            |row| row.get(0),
        )?;
        Ok(count as u32)
    }

    /// Get aggregate drill stats for a player.
    pub fn get_drill_stats(&self, player_id: &str) -> Result<DrillStats, DatabaseError> {
        let mut stmt = self.conn().prepare(
            "SELECT COUNT(*), SUM(CASE WHEN correct = 1 THEN 1 ELSE 0 END)
             FROM repertoire_drill_attempt WHERE player_id = ?1",
        )?;
        let (total_drills, total_correct): (i64, i64) = stmt
            .query_row(params![player_id], |row| {
                Ok((row.get(0)?, row.get::<_, Option<i64>>(1)?.unwrap_or(0)))
            })?;

        // Current streak
        let mut streak_stmt = self.conn().prepare(
            "SELECT correct FROM repertoire_drill_attempt
             WHERE player_id = ?1 ORDER BY attempted_at DESC",
        )?;
        let solved_iter = streak_stmt.query_map(params![player_id], |row| row.get::<_, bool>(0))?;
        let mut current_streak: u32 = 0;
        for solved in solved_iter {
            if solved? {
                current_streak += 1;
            } else {
                break;
            }
        }

        // Count distinct openings drilled
        let openings_drilled: i64 = self.conn().query_row(
            "SELECT COUNT(DISTINCT re.opening_id)
             FROM repertoire_drill_attempt rda
             JOIN repertoire_entry re ON re.id = rda.repertoire_entry_id
             WHERE rda.player_id = ?1",
            params![player_id],
            |row| row.get(0),
        )?;

        Ok(DrillStats {
            total_drills: total_drills as u32,
            total_correct: total_correct as u32,
            current_streak,
            openings_drilled: openings_drilled as u32,
        })
    }

    /// Bulk import openings with position tree generation.
    pub fn import_openings(&self, openings: &[Opening]) -> Result<usize, DatabaseError> {
        let tx = self.conn().unchecked_transaction()?;
        let mut count = 0;

        {
            let mut opening_stmt = tx.prepare(
                "INSERT OR IGNORE INTO opening (id, name, eco, color, description, moves, themes, difficulty)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )?;

            let mut pos_stmt = tx.prepare(
                "INSERT OR IGNORE INTO opening_position (id, opening_id, fen, move_index, parent_fen, move_uci, move_san)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )?;

            for opening in openings {
                let inserted = opening_stmt.execute(params![
                    opening.id,
                    opening.name,
                    opening.eco,
                    opening.color,
                    opening.description,
                    opening.moves,
                    opening.themes,
                    opening.difficulty as i64,
                ])?;
                count += inserted;

                // Build position tree from UCI moves
                if inserted > 0 {
                    let positions = build_position_tree(&opening.id, &opening.moves);
                    for pos in &positions {
                        let _ = pos_stmt.execute(params![
                            pos.id,
                            pos.opening_id,
                            pos.fen,
                            pos.move_index as i64,
                            pos.parent_fen,
                            pos.move_uci,
                            pos.move_san,
                        ]);
                    }
                }
            }
        }

        tx.commit()?;
        Ok(count)
    }

    /// Get the count of openings in the database.
    pub fn get_opening_count(&self) -> Result<u32, DatabaseError> {
        let count: i64 = self
            .conn()
            .query_row("SELECT COUNT(*) FROM opening", [], |row| row.get(0))?;
        Ok(count as u32)
    }
}

/// Parse opening UCI moves with shakmaty and generate position tree nodes.
fn build_position_tree(opening_id: &str, moves_str: &str) -> Vec<OpeningPosition> {
    let uci_moves: Vec<&str> = moves_str.split_whitespace().collect();
    if uci_moves.is_empty() {
        return Vec::new();
    }

    let mut positions = Vec::new();
    let mut chess = Chess::default();
    let mut parent_fen: Option<String> = None;

    for (i, uci_str) in uci_moves.iter().enumerate() {
        let current_fen = Fen::from_position(chess.clone(), EnPassantMode::Legal).to_string();

        let uci: UciMove = match uci_str.parse() {
            Ok(u) => u,
            Err(_) => break,
        };
        let legal_move = match uci.to_move(&chess) {
            Ok(m) => m,
            Err(_) => break,
        };

        // Get SAN notation before applying the move
        let san = San::from_move(&chess, &legal_move);
        let san_str = san.to_string();

        chess.play_unchecked(&legal_move);
        let after_fen = Fen::from_position(chess.clone(), EnPassantMode::Legal).to_string();

        positions.push(OpeningPosition {
            id: format!("{opening_id}_pos_{i}"),
            opening_id: opening_id.to_string(),
            fen: after_fen,
            move_index: i as u32,
            parent_fen: parent_fen.clone(),
            move_uci: uci_str.to_string(),
            move_san: san_str,
        });

        parent_fen = Some(current_fen);
    }

    positions
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../../migrations/001_initial.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../../migrations/005_repertoire.sql"))
            .unwrap();
        // Insert a test player
        conn.execute(
            "INSERT INTO player (id, display_name) VALUES ('p1', 'Test')",
            [],
        )
        .unwrap();
        Database::from_connection(conn)
    }

    #[test]
    fn build_position_tree_from_moves() {
        let positions = build_position_tree("italian", "e2e4 e7e5 g1f3 b8c6 f1c4");
        assert_eq!(positions.len(), 5);
        assert_eq!(positions[0].move_uci, "e2e4");
        assert_eq!(positions[0].move_san, "e4");
        assert_eq!(positions[0].move_index, 0);
        assert!(positions[0].parent_fen.is_none());
        assert_eq!(positions[1].move_san, "e5");
        assert!(positions[1].parent_fen.is_some());
        assert_eq!(positions[4].move_san, "Bc4");
    }

    #[test]
    fn import_and_query_openings() {
        let db = setup_test_db();
        let openings = vec![Opening {
            id: "italian".to_string(),
            name: "Italian Game".to_string(),
            eco: "C50".to_string(),
            color: "white".to_string(),
            description: "Classic opening".to_string(),
            moves: "e2e4 e7e5 g1f3 b8c6 f1c4".to_string(),
            themes: "open,classical".to_string(),
            difficulty: 1000,
        }];
        let count = db.import_openings(&openings).unwrap();
        assert_eq!(count, 1);

        // Query back
        let all = db.get_openings(&RepertoireFilter::default()).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "Italian Game");

        // Check positions
        let positions = db.get_opening_positions("italian").unwrap();
        assert_eq!(positions.len(), 5);
    }

    #[test]
    fn repertoire_entry_crud() {
        let db = setup_test_db();
        let openings = vec![Opening {
            id: "italian".to_string(),
            name: "Italian Game".to_string(),
            eco: "C50".to_string(),
            color: "white".to_string(),
            description: "".to_string(),
            moves: "e2e4 e7e5 g1f3 b8c6 f1c4".to_string(),
            themes: "".to_string(),
            difficulty: 1000,
        }];
        db.import_openings(&openings).unwrap();

        let entry = RepertoireEntry {
            id: "e1".to_string(),
            player_id: "p1".to_string(),
            opening_id: "italian".to_string(),
            position_fen: "starting_fen".to_string(),
            move_uci: "e2e4".to_string(),
            move_san: "e4".to_string(),
            notes: "".to_string(),
        };
        db.add_repertoire_entry(&entry).unwrap();

        let entries = db.get_repertoire_entries("p1", "italian").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].move_uci, "e2e4");

        // Upsert should update the move
        let updated = RepertoireEntry {
            id: "e1_new".to_string(),
            move_uci: "d2d4".to_string(),
            move_san: "d4".to_string(),
            ..entry
        };
        db.add_repertoire_entry(&updated).unwrap();
        let entries = db.get_repertoire_entries("p1", "italian").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].move_uci, "d2d4");

        // Remove
        db.remove_repertoire_entry(&entries[0].id).unwrap();
        let entries = db.get_repertoire_entries("p1", "italian").unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn filter_openings_by_color() {
        let db = setup_test_db();
        let openings = vec![
            Opening {
                id: "italian".to_string(),
                name: "Italian".to_string(),
                eco: "C50".to_string(),
                color: "white".to_string(),
                description: "".to_string(),
                moves: "e2e4 e7e5 g1f3".to_string(),
                themes: "".to_string(),
                difficulty: 1000,
            },
            Opening {
                id: "sicilian".to_string(),
                name: "Sicilian".to_string(),
                eco: "B20".to_string(),
                color: "black".to_string(),
                description: "".to_string(),
                moves: "e2e4 c7c5".to_string(),
                themes: "".to_string(),
                difficulty: 1200,
            },
        ];
        db.import_openings(&openings).unwrap();

        let white_only = db
            .get_openings(&RepertoireFilter {
                color: Some("white".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(white_only.len(), 1);
        assert_eq!(white_only[0].name, "Italian");
    }
}
