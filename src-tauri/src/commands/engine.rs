use tauri::{AppHandle, State};

use crate::engine::process::EngineProcess;
use crate::error::AppError;
use crate::models::engine::{EngineEvaluation, EngineMove};

#[tauri::command]
pub async fn start_engine(
    app: AppHandle,
    state: State<'_, tokio::sync::Mutex<EngineProcess>>,
) -> Result<bool, AppError> {
    let mut engine = state.lock().await;
    engine.start(&app).await?;
    Ok(true)
}

#[tauri::command]
pub async fn stop_engine(
    state: State<'_, tokio::sync::Mutex<EngineProcess>>,
) -> Result<(), AppError> {
    let mut engine = state.lock().await;
    engine.stop().await?;
    Ok(())
}

#[tauri::command]
pub async fn get_engine_move(
    fen: String,
    depth: Option<u32>,
    elo: Option<u32>,
    skill_level: Option<u8>,
    state: State<'_, tokio::sync::Mutex<EngineProcess>>,
) -> Result<EngineMove, AppError> {
    let mut engine = state.lock().await;
    let engine_move = engine.get_move(&fen, depth, elo, skill_level).await?;
    Ok(engine_move)
}

#[tauri::command]
pub async fn analyze_position(
    fen: String,
    depth: u32,
    state: State<'_, tokio::sync::Mutex<EngineProcess>>,
) -> Result<EngineEvaluation, AppError> {
    let mut engine = state.lock().await;
    let eval = engine.analyze(&fen, depth).await?;
    Ok(eval)
}
