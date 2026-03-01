use std::sync::Mutex;

use tauri::State;

use crate::error::AppError;
use crate::game::state::GameState;
use crate::models::chess::Position;
use crate::models::game::GameConfig;
use crate::models::game::GameRecord;

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
pub fn make_move(
    uci: String,
    state: State<'_, Mutex<GameState>>,
) -> Result<Position, AppError> {
    let mut game = state.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    game.make_move(&uci)?;
    Ok(game.to_position())
}

#[tauri::command]
pub fn resign(
    state: State<'_, Mutex<GameState>>,
) -> Result<GameRecord, AppError> {
    let mut game = state.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let record = game.resign()?;
    Ok(record)
}

#[tauri::command]
pub fn get_position(
    state: State<'_, Mutex<GameState>>,
) -> Result<Position, AppError> {
    let game = state.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    Ok(game.to_position())
}
