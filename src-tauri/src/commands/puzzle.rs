use std::path::PathBuf;
use std::sync::Mutex;

use tauri::State;

use crate::db::connection::Database;
use crate::error::{AppError, PuzzleError};
use crate::models::puzzle::{
    PuzzleAttempt, PuzzleFilter, PuzzleMoveResult, PuzzleSessionStats, PuzzleState,
};
use crate::puzzle::session;
use crate::puzzle::srs;
use crate::puzzle::PuzzleSessionState;
use crate::CurrentPlayerId;

#[tauri::command]
pub fn load_next_puzzle(
    filter: PuzzleFilter,
    db: State<'_, Mutex<Database>>,
    player_state: State<'_, CurrentPlayerId>,
    session: State<'_, Mutex<PuzzleSessionState>>,
) -> Result<PuzzleState, AppError> {
    let player_id = player_state.get()?;

    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;

    // Apply adaptive difficulty if player has a skill rating and no explicit custom range
    let filter = if filter.min_difficulty.is_none() && filter.max_difficulty.is_none() {
        let category = filter
            .category
            .as_ref()
            .map(|c| c.as_str())
            .unwrap_or("tactical");
        if let Some(skill) = db.get_skill_rating(&player_id, category)? {
            if skill.games_count >= 5 {
                let target = crate::assessment::difficulty_target_from_rating(skill.rating);
                PuzzleFilter {
                    min_difficulty: Some(target.min_rating),
                    max_difficulty: Some(target.max_rating),
                    ..filter
                }
            } else {
                filter
            }
        } else {
            filter
        }
    } else {
        filter
    };

    // Try SRS-due puzzles first, then new puzzles
    let puzzle = db
        .get_next_due_puzzle(&player_id, &filter)?
        .or(db.get_next_new_puzzle(&player_id, &filter)?)
        .ok_or(PuzzleError::NoPuzzlesAvailable)?;

    let (state, active) = session::start_puzzle(&puzzle)?;

    let mut session = session
        .inner()
        .lock()
        .map_err(|e| AppError::Lock(e.to_string()))?;
    session.puzzle = Some(active);

    Ok(state)
}

#[tauri::command]
pub fn get_puzzle_state(
    session: State<'_, Mutex<PuzzleSessionState>>,
) -> Result<Option<PuzzleState>, AppError> {
    let session = session.lock().map_err(|e| AppError::Lock(e.to_string()))?;

    let active = match &session.puzzle {
        Some(a) => a,
        None => return Ok(None),
    };

    let player_move_idx = if active.current_move_index > 0 {
        ((active.current_move_index - 1) / 2) as u32
    } else {
        0
    };

    // Count player moves
    let total_player_moves = active
        .solution_moves
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
        .count() as u32;

    Ok(Some(PuzzleState {
        puzzle: active.puzzle.clone(),
        start_fen: active.current_fen.clone(),
        player_color: active.player_color.clone(),
        legal_dests: active.legal_dests.clone(),
        total_player_moves,
        current_move_index: player_move_idx,
    }))
}

#[tauri::command]
pub fn submit_puzzle_move(
    uci: String,
    session: State<'_, Mutex<PuzzleSessionState>>,
) -> Result<PuzzleMoveResult, AppError> {
    let mut session = session.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let active = session.puzzle.as_mut().ok_or(PuzzleError::NoPuzzleActive)?;

    session::validate_move(active, &uci)
}

#[tauri::command]
pub fn request_puzzle_hint(
    session: State<'_, Mutex<PuzzleSessionState>>,
) -> Result<Option<String>, AppError> {
    let mut session = session.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let active = session.puzzle.as_mut().ok_or(PuzzleError::NoPuzzleActive)?;

    Ok(session::reveal_hint(active))
}

#[tauri::command]
pub fn abandon_puzzle(
    db: State<'_, Mutex<Database>>,
    player_state: State<'_, CurrentPlayerId>,
    session: State<'_, Mutex<PuzzleSessionState>>,
) -> Result<PuzzleMoveResult, AppError> {
    let player_id = player_state.get()?;

    let mut session = session.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let active = session.puzzle.as_ref().ok_or(PuzzleError::NoPuzzleActive)?;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let time_ms = now_ms.saturating_sub(active.start_time_ms);

    // Get previous SRS state
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let (prev_interval, prev_ease) = db
        .get_latest_srs(&player_id, &active.puzzle.id)?
        .unwrap_or((1.0, 2.5));
    let attempt_count = db.get_attempt_count(&player_id, &active.puzzle.id)? + 1;

    let quality = srs::quality_from_attempt(false, active.hints_revealed, time_ms);
    let srs_update = srs::compute_srs_update(prev_interval, prev_ease, quality, attempt_count);

    let attempt = PuzzleAttempt {
        id: uuid::Uuid::new_v4().to_string(),
        player_id,
        puzzle_id: active.puzzle.id.clone(),
        solved: false,
        time_ms,
        hints_used: active.hints_revealed,
        attempted_at: srs_update.next_review.clone(), // will be overridden
        srs_interval: srs_update.interval,
        srs_ease: srs_update.ease_factor,
        srs_next_review: srs_update.next_review,
    };
    db.save_puzzle_attempt(&attempt)?;

    // Build explanation with full solution
    let explanation = session::get_solution_explanation(&active.puzzle);
    let solution_moves: Vec<&str> = active.puzzle.solution_moves.split_whitespace().collect();
    let solution_text = format!(
        "{}\n\nSolution: {}",
        explanation,
        solution_moves.join(" → ")
    );

    // Clear active puzzle
    session.puzzle = None;

    Ok(PuzzleMoveResult {
        correct: false,
        is_complete: true,
        fen: None,
        legal_dests: None,
        last_move: None,
        current_move_index: 0,
        explanation: Some(solution_text),
    })
}

#[tauri::command]
pub fn save_puzzle_result(
    solved: bool,
    db: State<'_, Mutex<Database>>,
    player_state: State<'_, CurrentPlayerId>,
    session: State<'_, Mutex<PuzzleSessionState>>,
) -> Result<(), AppError> {
    let player_id = player_state.get()?;

    let mut session = session.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let active = session.puzzle.as_ref().ok_or(PuzzleError::NoPuzzleActive)?;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let time_ms = now_ms.saturating_sub(active.start_time_ms);

    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let (prev_interval, prev_ease) = db
        .get_latest_srs(&player_id, &active.puzzle.id)?
        .unwrap_or((1.0, 2.5));
    let attempt_count = db.get_attempt_count(&player_id, &active.puzzle.id)? + 1;

    let quality = srs::quality_from_attempt(solved, active.hints_revealed, time_ms);
    let srs_update = srs::compute_srs_update(prev_interval, prev_ease, quality, attempt_count);

    let attempt = PuzzleAttempt {
        id: uuid::Uuid::new_v4().to_string(),
        player_id,
        puzzle_id: active.puzzle.id.clone(),
        solved,
        time_ms,
        hints_used: active.hints_revealed,
        attempted_at: srs_update.next_review.clone(),
        srs_interval: srs_update.interval,
        srs_ease: srs_update.ease_factor,
        srs_next_review: srs_update.next_review,
    };
    db.save_puzzle_attempt(&attempt)?;

    // Update Glicko-2 skill rating for this puzzle's category
    let category = active.puzzle.category.as_str();
    let skill = db
        .get_skill_rating(&attempt.player_id, category)?
        .unwrap_or_else(|| {
            crate::models::assessment::SkillRating::default_for(&attempt.player_id, category)
        });
    let updated_skill =
        crate::assessment::glicko2::update_rating(&skill, active.puzzle.difficulty as f64, solved);
    db.upsert_skill_rating(&updated_skill)?;

    // Clear session
    session.puzzle = None;

    Ok(())
}

#[tauri::command]
pub fn get_puzzle_stats(
    db: State<'_, Mutex<Database>>,
    player_state: State<'_, CurrentPlayerId>,
) -> Result<PuzzleSessionStats, AppError> {
    let player_id = player_state.get()?;

    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let stats = db.get_puzzle_stats(&player_id)?;
    Ok(stats)
}

#[tauri::command]
pub fn import_puzzles_from_csv(
    path: String,
    min_rating: u32,
    max_rating: u32,
    db: State<'_, Mutex<Database>>,
) -> Result<usize, AppError> {
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let file_path = PathBuf::from(path);
    crate::puzzle::importer::import_lichess_csv(&file_path, &db, (min_rating, max_rating), 0)
}

#[tauri::command]
pub fn get_puzzle_themes(db: State<'_, Mutex<Database>>) -> Result<Vec<String>, AppError> {
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let themes = db.get_puzzle_themes()?;
    Ok(themes)
}
