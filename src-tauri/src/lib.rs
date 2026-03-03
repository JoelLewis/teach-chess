mod assessment;
mod coaching;
mod commands;
pub mod config;
mod db;
mod engine;
mod error;
mod game;
mod heuristics;
mod llm;
mod models;
mod opponent;
mod puzzle;
mod repertoire;

use tauri::Manager;

/// Stores the current player's ID so game commands can reference it.
pub struct CurrentPlayerId(std::sync::Mutex<Option<String>>);

impl CurrentPlayerId {
    pub fn new() -> Self {
        Self(std::sync::Mutex::new(None))
    }

    pub fn get(&self) -> Result<String, crate::error::AppError> {
        Ok(self
            .0
            .lock()
            .map_err(|e| crate::error::AppError::Lock(e.to_string()))?
            .clone()
            .unwrap_or_default())
    }

    pub fn set(&self, id: String) -> Result<(), crate::error::AppError> {
        let mut lock = self
            .0
            .lock()
            .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
        *lock = Some(id);
        Ok(())
    }
}

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
            app.manage(CurrentPlayerId::new());

            // Load app config (theme, audio preferences)
            let app_data_dir = app
                .handle()
                .path()
                .app_data_dir()
                .expect("app data dir");
            app.manage(std::sync::Mutex::new(
                config::AppConfigState::load(&app_data_dir),
            ));
            app.manage(std::sync::Mutex::new(
                puzzle::PuzzleSessionState::default(),
            ));
            app.manage(std::sync::Mutex::new(
                repertoire::RepertoireSessionState::default(),
            ));

            // Initialize LLM state (feature-gated)
            #[cfg(feature = "llm")]
            {
                let app_data_dir = app
                    .handle()
                    .path()
                    .app_data_dir()
                    .expect("app data dir");
                app.manage(llm::LlmState::new(app_data_dir));
            }

            // Import bundled starter data if tables are empty
            {
                let db_lock = app.state::<std::sync::Mutex<db::connection::Database>>();
                let db = db_lock.lock().expect("DB lock for starter import");
                if let Err(e) = puzzle::importer::import_starter_puzzles_if_empty(&db) {
                    tracing::warn!("Failed to import starter puzzles: {e}");
                }
                if let Err(e) = repertoire::importer::import_starter_openings_if_empty(&db) {
                    tracing::warn!("Failed to import starter openings: {e}");
                }
            }

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
            commands::review::get_critical_moments,
            commands::review::get_pattern_summary,
            commands::review::get_study_suggestions,
            commands::heuristics::analyze_heuristics,
            commands::llm::get_llm_status,
            commands::llm::get_available_models,
            commands::llm::download_model,
            commands::llm::generate_coaching,
            commands::coaching::evaluate_player_move,
            commands::coaching::analyze_pre_move_hints,
            commands::puzzle::load_next_puzzle,
            commands::puzzle::get_puzzle_state,
            commands::puzzle::submit_puzzle_move,
            commands::puzzle::request_puzzle_hint,
            commands::puzzle::abandon_puzzle,
            commands::puzzle::save_puzzle_result,
            commands::puzzle::get_puzzle_stats,
            commands::puzzle::import_puzzles_from_csv,
            commands::puzzle::get_puzzle_themes,
            commands::repertoire::get_openings,
            commands::repertoire::get_opening_detail,
            commands::repertoire::get_repertoire,
            commands::repertoire::add_to_repertoire,
            commands::repertoire::remove_from_repertoire,
            commands::repertoire::start_repertoire_drill,
            commands::repertoire::submit_drill_move,
            commands::repertoire::get_drill_stats,
            commands::repertoire::import_openings_from_json,
            commands::assessment::get_skill_profile,
            commands::assessment::get_skill_rating,
            commands::assessment::get_difficulty_target,
            commands::opponent::get_opponent_move,
            commands::opponent::resolve_personality,
            commands::dashboard::get_dashboard_data,
            commands::dashboard::check_adaptive_difficulty,
            commands::theme::get_theme,
            commands::theme::set_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
