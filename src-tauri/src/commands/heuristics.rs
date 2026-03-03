use crate::error::AppError;
use crate::models::heuristics::CoachingContext;

#[tauri::command]
pub fn analyze_heuristics(fen: String) -> Result<CoachingContext, AppError> {
    let chess = crate::game::parse_fen(&fen)?;
    Ok(crate::heuristics::analyze_position(&chess))
}
