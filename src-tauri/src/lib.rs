// `assessment`, `game`, `heuristics`, `llm`, and `models` are public so
// integration tests (e.g. the real-model grounding suite) can build prompts
// from positions and rank contexts.
pub mod assessment;
mod coaching;
mod commands;
pub mod config;
mod db;
mod engine;
mod error;
pub mod game;
pub mod heuristics;
pub mod llm;
pub mod models;
mod opponent;
mod puzzle;
mod repertoire;
mod srs;

use tauri::Manager;

/// Stores the current player's ID so game commands can reference it.
pub struct CurrentPlayerId(std::sync::Mutex<Option<String>>);

impl Default for CurrentPlayerId {
    fn default() -> Self {
        Self(std::sync::Mutex::new(None))
    }
}

impl CurrentPlayerId {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self) -> Result<String, crate::error::AppError> {
        self.0
            .lock()
            .map_err(|e| crate::error::AppError::Lock(e.to_string()))?
            .clone()
            .ok_or(crate::error::AppError::NotInitialized)
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

/// Collects every registered command with its Specta types.
///
/// Single source of truth for both the invoke handler and the generated
/// TypeScript bindings (`src/lib/api/bindings.ts`).
fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new()
        .error_handling(tauri_specta::ErrorHandlingMode::Result)
        // i64/u64/usize cross the IPC boundary as JSON numbers, so export them
        // as `number` (matching the previous hand-written types).
        .dangerously_cast_bigints_to_number()
        // Export f32/f64 as plain `number` instead of `number | null`
        // (the null only occurs for NaN/Infinity, which these APIs never produce).
        .semantic_types(
            specta_typescript::semantic::Configuration::default().enable_lossless_floats(),
        )
        .commands(tauri_specta::collect_commands![
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
            commands::review::get_game,
            commands::review::get_critical_moments,
            commands::review::get_pattern_summary,
            commands::review::get_study_suggestions,
            commands::heuristics::analyze_heuristics,
            commands::llm::get_llm_status,
            commands::llm::get_available_models,
            commands::llm::download_model,
            commands::llm::generate_coaching,
            commands::llm::generate_game_summary,
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
}

#[cfg(any(debug_assertions, test))]
fn export_typescript_bindings(builder: &tauri_specta::Builder<tauri::Wry>) {
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/lib/api/bindings.ts",
        )
        .expect("failed to export typescript bindings");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = specta_builder();

    #[cfg(debug_assertions)]
    export_typescript_bindings(&specta_builder);

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("chess_mentor=debug".parse().unwrap()),
        )
        .init();

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_shell::init());

    // Dev-only playtest socket (agent driving). Compiled in only with
    // `--features mcp` (`pnpm playtest`); never part of default features.
    #[cfg(feature = "mcp")]
    {
        builder = builder.plugin(tauri_plugin_mcp::init_with_config(
            tauri_plugin_mcp::PluginConfig::new("ChessMentor".to_string())
                .start_socket_server(true)
                .socket_path("/tmp/chessmentor-mcp.sock".into()),
        ));
    }

    let app = builder
        .setup(|app| {
            let app_handle = app.handle().clone();
            let db = db::connection::Database::open(&app_handle)?;
            app.manage(std::sync::Mutex::new(db));
            app.manage(std::sync::Mutex::new(game::state::GameState::default()));
            app.manage(tokio::sync::Mutex::new(
                engine::process::EngineProcess::default(),
            ));
            app.manage(CurrentPlayerId::new());

            // Load app config (theme, audio preferences)
            let app_data_dir = app.handle().path().app_data_dir().expect("app data dir");
            app.manage(std::sync::Mutex::new(config::AppConfigState::load(
                &app_data_dir,
            )));
            app.manage(std::sync::Mutex::new(puzzle::PuzzleSessionState::default()));
            app.manage(std::sync::Mutex::new(
                repertoire::RepertoireSessionState::default(),
            ));

            // Initialize LLM state (feature-gated)
            #[cfg(feature = "llm")]
            {
                // Honor the deprecated CHESS_MENTOR_DEVICE variable before
                // any inference thread reads SENSEI_LLM_DEVICE.
                llm::llm_support::apply_legacy_device_env_fallback();

                let app_data_dir = app.handle().path().app_data_dir().expect("app data dir");
                let resource_dir = app
                    .handle()
                    .path()
                    .resource_dir()
                    .ok()
                    .map(|d| d.join("models"));
                app.manage(llm::LlmState::new(app_data_dir, resource_dir));

                // Warm the model in the background so the first coaching
                // request doesn't pay the ~15s load. Failure is non-fatal —
                // requests fall back to lazy init / templates.
                let warm_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let llm_state = warm_handle.state::<llm::LlmState>();
                    if llm_state.store.is_available(llm::GEMMA4_E2B.spec) {
                        match llm_state.ensure_channel().await {
                            Ok(()) => tracing::info!("LLM warm-up started at app startup"),
                            Err(e) => tracing::warn!("LLM warm-up failed (non-fatal): {e}"),
                        }
                    } else {
                        tracing::info!("LLM model not available — skipping warm-up");
                    }
                });
            }

            // Clean up expired coaching cache entries
            {
                let db_lock = app.state::<std::sync::Mutex<db::connection::Database>>();
                let db = db_lock.lock().expect("DB lock for cache cleanup");
                match db.cleanup_expired_cache() {
                    Ok(n) if n > 0 => {
                        tracing::info!("Cleaned up {n} expired coaching cache entries")
                    }
                    Ok(_) => {}
                    Err(e) => tracing::warn!("Failed to clean up coaching cache: {e}"),
                }
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
        .invoke_handler(specta_builder.invoke_handler())
        .build(tauri::generate_context!())
        .expect("error building tauri application");

    app.run(|app_handle: &tauri::AppHandle, event: tauri::RunEvent| {
        if let tauri::RunEvent::Exit = event {
            let engine_state =
                app_handle.state::<tokio::sync::Mutex<engine::process::EngineProcess>>();
            let lock_result = engine_state.try_lock();
            if let Ok(mut engine) = lock_result {
                // Use a new runtime since we're in a sync callback during shutdown
                if let Ok(rt) = tokio::runtime::Runtime::new() {
                    let _ = rt.block_on(engine.stop());
                }
                tracing::info!("Engine stopped on app exit");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::error::AppError;

    #[test]
    fn current_player_id_returns_not_initialized_error() {
        let current_player = super::CurrentPlayerId::new();

        assert!(matches!(
            current_player.get(),
            Err(AppError::NotInitialized)
        ));
    }

    /// Regenerates `src/lib/api/bindings.ts`. Run via `cargo test export_bindings`.
    #[test]
    fn export_bindings() {
        super::export_typescript_bindings(&super::specta_builder());
    }
}
