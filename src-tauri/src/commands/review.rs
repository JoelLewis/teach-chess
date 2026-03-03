use std::sync::Mutex;

use tauri::{AppHandle, State};

use crate::coaching;
use crate::db::connection::Database;
use crate::engine::{eval, process::EngineProcess};
use crate::error::AppError;
use crate::models::engine::{CriticalMoment, MoveEvaluation, PatternSummary, StudySuggestion};
use crate::models::game::GameRecord;

#[tauri::command]
pub async fn get_game_review(
    game_id: String,
    depth: u32,
    app: AppHandle,
    db: State<'_, Mutex<Database>>,
    engine: State<'_, tokio::sync::Mutex<EngineProcess>>,
) -> Result<Vec<MoveEvaluation>, AppError> {
    let game = {
        let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db.get_game(&game_id)?
    };

    let mut engine = engine.lock().await;
    let evals = engine.review_game(&game, depth, Some(&app)).await?;

    // Save annotations to database
    {
        let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        let _ = db.save_move_annotations(&game_id, &evals);
    }

    Ok(evals)
}

#[tauri::command]
pub fn get_game_history(
    limit: u32,
    offset: u32,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<GameRecord>, AppError> {
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let games = db.get_game_history(limit, offset)?;
    Ok(games)
}

#[tauri::command]
pub fn get_critical_moments(
    evaluations: Vec<MoveEvaluation>,
    is_player_white: bool,
) -> Vec<CriticalMoment> {
    eval::find_critical_moments(&evaluations, is_player_white)
}

#[tauri::command]
pub fn get_pattern_summary(
    evaluations: Vec<MoveEvaluation>,
    is_player_white: bool,
) -> PatternSummary {
    coaching::generate_pattern_summary(&evaluations, is_player_white)
}

#[tauri::command]
pub fn get_study_suggestions(summary: PatternSummary) -> Vec<StudySuggestion> {
    coaching::generate_study_suggestions(&summary)
}
