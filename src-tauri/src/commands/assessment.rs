use std::sync::Mutex;

use tauri::State;

use crate::db::connection::Database;
use crate::error::AppError;
use crate::models::assessment::{DifficultyTarget, SkillProfile, SkillRating};
use crate::CurrentPlayerId;

#[tauri::command]
pub fn get_skill_profile(
    db: State<'_, Mutex<Database>>,
    player_state: State<'_, CurrentPlayerId>,
) -> Result<SkillProfile, AppError> {
    let player_id = player_state.get()?;
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db.get_skill_profile(&player_id).map_err(Into::into)
}

#[tauri::command]
pub fn get_skill_rating(
    category: String,
    db: State<'_, Mutex<Database>>,
    player_state: State<'_, CurrentPlayerId>,
) -> Result<Option<SkillRating>, AppError> {
    let player_id = player_state.get()?;
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db.get_skill_rating(&player_id, &category)
        .map_err(Into::into)
}

#[tauri::command]
pub fn get_difficulty_target(
    category: String,
    db: State<'_, Mutex<Database>>,
    player_state: State<'_, CurrentPlayerId>,
) -> Result<DifficultyTarget, AppError> {
    let player_id = player_state.get()?;
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;

    let rating = db.get_skill_rating(&player_id, &category)?;
    let r = rating.map(|r| r.rating).unwrap_or(1200.0);

    Ok(crate::assessment::difficulty_target_from_rating(r))
}
