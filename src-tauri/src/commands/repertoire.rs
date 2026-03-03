use std::sync::Mutex;

use tauri::State;

use crate::db::connection::Database;
use crate::error::{AppError, RepertoireError};
use crate::models::repertoire::{
    DrillAttempt, DrillMoveResult, DrillState, DrillStats, Opening, OpeningPosition,
    RepertoireEntry, RepertoireFilter,
};
use crate::puzzle::srs;
use crate::repertoire::session;
use crate::repertoire::RepertoireSessionState;
use crate::CurrentPlayerId;

#[tauri::command]
pub fn get_openings(
    filter: RepertoireFilter,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<Opening>, AppError> {
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db.get_openings(&filter).map_err(Into::into)
}

#[tauri::command]
pub fn get_opening_detail(
    opening_id: String,
    db: State<'_, Mutex<Database>>,
) -> Result<(Opening, Vec<OpeningPosition>), AppError> {
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let opening = db
        .get_opening(&opening_id)?
        .ok_or_else(|| RepertoireError::OpeningNotFound(opening_id.clone()))?;
    let positions = db.get_opening_positions(&opening_id)?;
    Ok((opening, positions))
}

#[tauri::command]
pub fn get_repertoire(
    opening_id: String,
    db: State<'_, Mutex<Database>>,
    player_state: State<'_, CurrentPlayerId>,
) -> Result<Vec<RepertoireEntry>, AppError> {
    let player_id = player_state.get()?;
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db.get_repertoire_entries(&player_id, &opening_id)
        .map_err(Into::into)
}

#[tauri::command]
pub fn add_to_repertoire(
    opening_id: String,
    position_fen: String,
    move_uci: String,
    move_san: String,
    db: State<'_, Mutex<Database>>,
    player_state: State<'_, CurrentPlayerId>,
) -> Result<(), AppError> {
    let player_id = player_state.get()?;
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;

    let entry = RepertoireEntry {
        id: uuid::Uuid::new_v4().to_string(),
        player_id,
        opening_id,
        position_fen,
        move_uci,
        move_san,
        notes: String::new(),
    };
    db.add_repertoire_entry(&entry)?;
    Ok(())
}

#[tauri::command]
pub fn remove_from_repertoire(
    entry_id: String,
    db: State<'_, Mutex<Database>>,
) -> Result<(), AppError> {
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db.remove_repertoire_entry(&entry_id)?;
    Ok(())
}

#[tauri::command]
pub fn start_repertoire_drill(
    opening_id: String,
    db: State<'_, Mutex<Database>>,
    player_state: State<'_, CurrentPlayerId>,
    session_state: State<'_, Mutex<RepertoireSessionState>>,
) -> Result<DrillState, AppError> {
    let player_id = player_state.get()?;

    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;

    let opening = db
        .get_opening(&opening_id)?
        .ok_or_else(|| RepertoireError::OpeningNotFound(opening_id.clone()))?;

    // Get entries to drill: SRS-due first, then new entries
    let mut entries = Vec::new();

    // Collect SRS-due entries
    while let Some(entry) = db.get_next_due_drill_entry(&player_id, &opening_id)? {
        // Avoid duplicates
        if entries.iter().any(|e: &RepertoireEntry| e.id == entry.id) {
            break;
        }
        entries.push(entry);
        if entries.len() >= 10 {
            break;
        }
    }

    // Fill with new entries if we don't have enough
    while entries.len() < 10 {
        match db.get_next_new_drill_entry(&player_id, &opening_id)? {
            Some(entry) if !entries.iter().any(|e: &RepertoireEntry| e.id == entry.id) => {
                entries.push(entry);
            }
            _ => break,
        }
    }

    // Fallback: get all entries for this opening
    if entries.is_empty() {
        entries = db.get_repertoire_entries(&player_id, &opening_id)?;
    }

    if entries.is_empty() {
        return Err(RepertoireError::NoRepertoireEntries.into());
    }

    let (state, active) = session::start_drill(&opening, entries)?;

    let mut session = session_state
        .inner()
        .lock()
        .map_err(|e| AppError::Lock(e.to_string()))?;
    session.drill = Some(active);

    Ok(state)
}

#[tauri::command]
pub fn submit_drill_move(
    uci: String,
    db: State<'_, Mutex<Database>>,
    player_state: State<'_, CurrentPlayerId>,
    session_state: State<'_, Mutex<RepertoireSessionState>>,
) -> Result<DrillMoveResult, AppError> {
    let player_id = player_state.get()?;

    let mut session = session_state
        .lock()
        .map_err(|e| AppError::Lock(e.to_string()))?;
    let active = session
        .drill
        .as_mut()
        .ok_or(RepertoireError::NoDrillActive)?;

    let time_ms = session::get_entry_elapsed_ms(active);
    let result = session::validate_drill_move(active, &uci)?;

    // Save drill attempt with SRS
    let entry_id = active.entries[active.current_index].id.clone();
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;

    let (prev_interval, prev_ease) = db
        .get_latest_drill_srs(&player_id, &entry_id)?
        .unwrap_or((1.0, 2.5));
    let attempt_count = db.get_drill_attempt_count(&player_id, &entry_id)? + 1;

    let quality = session::drill_quality(result.correct, time_ms);
    let srs_update = srs::compute_srs_update(prev_interval, prev_ease, quality, attempt_count);

    let attempt = DrillAttempt {
        id: uuid::Uuid::new_v4().to_string(),
        player_id,
        repertoire_entry_id: entry_id,
        correct: result.correct,
        time_ms,
        srs_interval: srs_update.interval,
        srs_ease: srs_update.ease_factor,
        srs_next_review: srs_update.next_review,
    };
    db.save_drill_attempt(&attempt)?;

    // If correct, advance to next entry
    if result.correct && !result.is_complete {
        let _ = session::advance_drill(active);
    }

    // If complete, clear session
    if result.is_complete {
        session.drill = None;
    }

    Ok(result)
}

#[tauri::command]
pub fn get_drill_stats(
    db: State<'_, Mutex<Database>>,
    player_state: State<'_, CurrentPlayerId>,
) -> Result<DrillStats, AppError> {
    let player_id = player_state.get()?;
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db.get_drill_stats(&player_id).map_err(Into::into)
}

#[tauri::command]
pub fn import_openings_from_json(
    path: String,
    db: State<'_, Mutex<Database>>,
) -> Result<usize, AppError> {
    let json_str = std::fs::read_to_string(&path)
        .map_err(|e| RepertoireError::ImportError(format!("Cannot read file: {e}")))?;
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    crate::repertoire::importer::import_openings_json(&json_str, &db)
}
