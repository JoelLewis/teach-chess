use crate::error::DatabaseError;
use crate::models::engine::MoveEvaluation;
use crate::models::game::GameRecord;

use super::connection::Database;

impl Database {
    pub fn save_game(&self, record: &GameRecord) -> Result<(), DatabaseError> {
        let player_color = match record.player_color {
            crate::models::chess::Color::White => "white",
            crate::models::chess::Color::Black => "black",
        };

        self.conn().execute(
            "INSERT INTO game (id, player_id, pgn, result, player_color, engine_elo, move_count, started_at, ended_at, time_control)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                record.id,
                record.player_id,
                record.pgn,
                record.result,
                player_color,
                record.engine_elo,
                record.move_count,
                record.started_at,
                record.ended_at,
                record.time_control,
            ],
        )?;

        // Increment player's games_played count
        self.conn().execute(
            "UPDATE player SET games_played = games_played + 1 WHERE id = ?1",
            [&record.player_id],
        )?;

        Ok(())
    }

    pub fn get_game(&self, game_id: &str) -> Result<GameRecord, DatabaseError> {
        self.conn()
            .query_row(
                "SELECT id, player_id, pgn, result, player_color, engine_elo, move_count, started_at, ended_at, time_control
                 FROM game WHERE id = ?1",
                [game_id],
                |row| {
                    let color_str: String = row.get(4)?;
                    let player_color = if color_str == "white" {
                        crate::models::chess::Color::White
                    } else {
                        crate::models::chess::Color::Black
                    };

                    Ok(GameRecord {
                        id: row.get(0)?,
                        player_id: row.get(1)?,
                        pgn: row.get(2)?,
                        result: row.get(3)?,
                        player_color,
                        engine_elo: row.get(5)?,
                        move_count: row.get(6)?,
                        started_at: row.get(7)?,
                        ended_at: row.get(8)?,
                        time_control: row.get(9)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DatabaseError::GameNotFound(game_id.to_string())
                }
                other => DatabaseError::Sqlite(other),
            })
    }

    pub fn get_game_history(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<GameRecord>, DatabaseError> {
        let mut stmt = self.conn().prepare(
            "SELECT id, player_id, pgn, result, player_color, engine_elo, move_count, started_at, ended_at, time_control
             FROM game ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
        )?;

        let games = stmt
            .query_map(rusqlite::params![limit, offset], |row| {
                let color_str: String = row.get(4)?;
                let player_color = if color_str == "white" {
                    crate::models::chess::Color::White
                } else {
                    crate::models::chess::Color::Black
                };

                Ok(GameRecord {
                    id: row.get(0)?,
                    player_id: row.get(1)?,
                    pgn: row.get(2)?,
                    result: row.get(3)?,
                    player_color,
                    engine_elo: row.get(5)?,
                    move_count: row.get(6)?,
                    started_at: row.get(7)?,
                    ended_at: row.get(8)?,
                    time_control: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(games)
    }

    pub fn save_move_annotations(
        &self,
        game_id: &str,
        evals: &[MoveEvaluation],
    ) -> Result<(), DatabaseError> {
        let tx = self.conn().unchecked_transaction()?;

        for eval in evals {
            let (eval_before_cp, eval_before_mate) = match &eval.eval_before {
                Some(crate::models::engine::Score::Cp { value }) => (Some(*value), None),
                Some(crate::models::engine::Score::Mate { moves }) => (None, Some(*moves)),
                None => (None, None),
            };

            let (eval_after_cp, eval_after_mate) = match &eval.eval_after {
                Some(crate::models::engine::Score::Cp { value }) => (Some(*value), None),
                Some(crate::models::engine::Score::Mate { moves }) => (None, Some(*moves)),
                None => (None, None),
            };

            let classification = eval.classification.map(|c| format!("{c:?}").to_lowercase());
            let pv_json = serde_json::to_string(&eval.pv).ok();

            tx.execute(
                "INSERT INTO move_annotation (id, game_id, move_number, is_white, fen_before, player_move_uci, player_move_san, engine_best_uci, engine_best_san, eval_before_cp, eval_after_cp, eval_before_mate, eval_after_mate, classification, depth, pv_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                rusqlite::params![
                    uuid::Uuid::new_v4().to_string(),
                    game_id,
                    eval.move_number,
                    eval.is_white,
                    eval.fen_before,
                    eval.player_move_uci,
                    eval.player_move_san,
                    eval.engine_best_uci,
                    eval.engine_best_san,
                    eval_before_cp,
                    eval_after_cp,
                    eval_before_mate,
                    eval_after_mate,
                    classification,
                    eval.depth,
                    pv_json,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }
}
