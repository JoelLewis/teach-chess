use crate::config::{AppConfigState, Theme};
use crate::error::AppError;
use tauri::State;

#[tauri::command]
pub fn get_theme(
    config: State<'_, std::sync::Mutex<AppConfigState>>,
) -> Result<String, AppError> {
    let lock = config
        .lock()
        .map_err(|e| AppError::Lock(e.to_string()))?;
    Ok(lock.get_theme().to_string())
}

#[tauri::command]
pub fn set_theme(
    theme: String,
    config: State<'_, std::sync::Mutex<AppConfigState>>,
) -> Result<(), AppError> {
    let parsed = match theme.as_str() {
        "study" => Theme::Study,
        "grid" => Theme::Grid,
        _ => return Err(AppError::Config(format!("Unknown theme: {theme}"))),
    };

    let mut lock = config
        .lock()
        .map_err(|e| AppError::Lock(e.to_string()))?;
    lock.set_theme(parsed)
        .map_err(AppError::Config)?;

    Ok(())
}
