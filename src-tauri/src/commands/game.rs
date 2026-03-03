use std::sync::Mutex;

use tauri::State;

use crate::db::connection::Database;
use crate::error::AppError;
use crate::game::state::GameState;
use crate::models::chess::Position;
use crate::models::game::GameConfig;
use crate::models::game::GameRecord;
use crate::CurrentPlayerId;

#[tauri::command]
pub fn new_game(
    config: GameConfig,
    state: State<'_, Mutex<GameState>>,
) -> Result<Position, AppError> {
    let mut game = state.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    game.new_game(config);
    Ok(game.to_position())
}

#[tauri::command]
pub fn make_move(uci: String, state: State<'_, Mutex<GameState>>) -> Result<Position, AppError> {
    let mut game = state.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    game.make_move(&uci)?;
    Ok(game.to_position())
}

#[tauri::command]
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
    let _ = db.save_game(&record);

    Ok(record)
}

#[tauri::command]
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
pub fn get_position(state: State<'_, Mutex<GameState>>) -> Result<Position, AppError> {
    let game = state.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    Ok(game.to_position())
}
