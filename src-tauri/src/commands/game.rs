use std::sync::Mutex;

use tauri::State;

use crate::CurrentPlayerId;
use crate::db::connection::Database;
use crate::engine::process::EngineProcess;
use crate::error::AppError;
use crate::game::state::GameState;
use crate::models::chess::Position;
use crate::models::game::GameConfig;
use crate::models::game::GameRecord;

fn save_resigned_game(db: &Database, record: &GameRecord) -> Result<(), AppError> {
    db.save_game(record)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn new_game(
    config: GameConfig,
    state: State<'_, Mutex<GameState>>,
    engine_state: State<'_, tokio::sync::Mutex<EngineProcess>>,
) -> Result<Position, AppError> {
    // Reset engine hash tables for the new game (best-effort — engine may not be running yet)
    {
        let mut engine = engine_state.lock().await;
        if let Err(e) = engine.new_game().await {
            tracing::debug!("Engine new_game skipped (not yet running): {e}");
        }
    }

    let mut game = state.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    game.new_game(config);
    Ok(game.to_position())
}

#[tauri::command]
#[specta::specta]
pub fn make_move(uci: String, state: State<'_, Mutex<GameState>>) -> Result<Position, AppError> {
    let mut game = state.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    game.make_move(&uci)?;
    Ok(game.to_position())
}

#[tauri::command]
#[specta::specta]
pub fn resign(
    state: State<'_, Mutex<GameState>>,
    db: State<'_, Mutex<Database>>,
    current_player: State<'_, CurrentPlayerId>,
) -> Result<GameRecord, AppError> {
    let player_id = current_player.get()?;

    let mut game = state.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let mut record = game.resign()?;
    record.player_id = player_id;

    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    save_resigned_game(&db, &record)?;

    Ok(record)
}

#[tauri::command]
#[specta::specta]
pub fn save_completed_game(
    state: State<'_, Mutex<GameState>>,
    db: State<'_, Mutex<Database>>,
    current_player: State<'_, CurrentPlayerId>,
) -> Result<GameRecord, AppError> {
    let player_id = current_player.get()?;

    let game = state.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let mut record = game.complete_game()?;
    record.player_id = player_id;

    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db.save_game(&record)?;

    Ok(record)
}

#[tauri::command]
#[specta::specta]
pub fn get_position(state: State<'_, Mutex<GameState>>) -> Result<Position, AppError> {
    let game = state.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    Ok(game.to_position())
}

#[cfg(test)]
mod tests {
    use super::save_resigned_game;
    use crate::db::connection::Database;
    use crate::error::AppError;
    use crate::models::chess::Color;
    use crate::models::game::GameRecord;

    #[test]
    fn resignation_save_failure_is_propagated() {
        let db = Database::open_in_memory().unwrap();
        let record = GameRecord {
            id: "resignation-test".to_string(),
            player_id: "missing-player".to_string(),
            pgn: String::new(),
            result: "resign".to_string(),
            player_color: Color::White,
            engine_elo: 1350,
            move_count: 0,
            started_at: "1767225600".to_string(),
            ended_at: None,
            time_control: String::new(),
            opponent_personality: None,
            teaching_mode: false,
        };

        assert!(matches!(
            save_resigned_game(&db, &record),
            Err(AppError::Database(_))
        ));
    }
}
