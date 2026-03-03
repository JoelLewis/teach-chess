use std::sync::Mutex;

use tauri::State;

use crate::db::connection::Database;
use crate::error::AppError;
use crate::models::player::{Player, PlayerSettings};
use crate::CurrentPlayerId;

#[tauri::command]
pub fn get_or_create_player(
    display_name: String,
    db: State<'_, Mutex<Database>>,
    current_player: State<'_, CurrentPlayerId>,
) -> Result<Player, AppError> {
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let player = db.get_or_create_player(&display_name)?;

    // Store current player ID for game commands
    let _ = current_player.set(player.id.clone());

    Ok(player)
}

#[tauri::command]
pub fn update_player_settings(
    player_id: String,
    settings: PlayerSettings,
    db: State<'_, Mutex<Database>>,
) -> Result<Player, AppError> {
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let player = db.update_player_settings(&player_id, &settings)?;
    Ok(player)
}
