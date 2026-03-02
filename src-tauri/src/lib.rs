mod commands;
mod db;
mod engine;
mod error;
mod game;
mod models;

use tauri::Manager;

/// Stores the current player's ID so game commands can reference it.
pub struct CurrentPlayerId(pub std::sync::Mutex<Option<String>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("chess_mentor=debug".parse().unwrap()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let db = db::connection::Database::open(&app_handle)?;
            app.manage(std::sync::Mutex::new(db));
            app.manage(std::sync::Mutex::new(
                game::state::GameState::default(),
            ));
            app.manage(tokio::sync::Mutex::new(
                engine::process::EngineProcess::default(),
            ));
            app.manage(CurrentPlayerId(std::sync::Mutex::new(None)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::game::new_game,
            commands::game::make_move,
            commands::game::resign,
            commands::game::save_completed_game,
            commands::game::get_position,
            commands::engine::start_engine,
            commands::engine::stop_engine,
            commands::engine::get_engine_move,
            commands::engine::analyze_position,
            commands::player::get_or_create_player,
            commands::player::update_player_settings,
            commands::review::get_game_review,
            commands::review::get_game_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
