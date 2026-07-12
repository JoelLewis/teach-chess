use serde::Serialize;

#[cfg(feature = "llm")]
use tauri::Manager;

use crate::llm::{CoachingResponse, CoachingSource};
use crate::models::heuristics::CoachingContext;

/// Serialized theme names (camelCase) from a coaching context, for the cache key.
fn theme_names(ctx: Option<&CoachingContext>) -> Vec<String> {
    ctx.map(|c| {
        c.themes
            .iter()
            .filter_map(|t| match serde_json::to_value(t) {
                Ok(serde_json::Value::String(s)) => Some(s),
                _ => None,
            })
            .collect()
    })
    .unwrap_or_default()
}

/// Status of the LLM subsystem
#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LlmStatus {
    pub available: bool,
    pub model_loaded: bool,
    pub model_id: Option<String>,
    pub mode: String,
    pub device: String,
    pub bundled: bool,
}

/// Download/availability status of a single model
#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub id: String,
    pub display_name: String,
    pub downloaded: bool,
    pub bundled: bool,
    pub file_size_mb: u32,
    pub ram_requirement_mb: u32,
    pub system_memory_mb: u32,
    pub available_memory_mb: u32,
}

/// Detect total system memory in MB using the sysinfo crate.
/// This is sandbox-compatible (no subprocess spawning).
#[cfg(feature = "llm")]
fn get_system_memory_mb() -> u32 {
    use sysinfo::System;
    let sys = System::new_with_specifics(
        sysinfo::RefreshKind::nothing().with_memory(sysinfo::MemoryRefreshKind::everything()),
    );
    (sys.total_memory() / (1024 * 1024)) as u32
}

/// Detect available (free) system memory in MB using the sysinfo crate.
/// This is sandbox-compatible (no subprocess spawning).
#[cfg(feature = "llm")]
fn get_available_memory_mb() -> u32 {
    use sysinfo::System;
    let sys = System::new_with_specifics(
        sysinfo::RefreshKind::nothing().with_memory(sysinfo::MemoryRefreshKind::everything()),
    );
    (sys.available_memory() / (1024 * 1024)) as u32
}

#[tauri::command]
#[specta::specta]
pub async fn get_llm_status(app: tauri::AppHandle) -> Result<LlmStatus, crate::error::AppError> {
    #[cfg(not(feature = "llm"))]
    {
        let _ = &app;
        Ok(LlmStatus {
            available: false,
            model_loaded: false,
            model_id: None,
            mode: "template".to_string(),
            device: "cpu".to_string(),
            bundled: false,
        })
    }

    #[cfg(feature = "llm")]
    {
        use crate::llm::GEMMA4_E2B;

        let llm_state = app.state::<crate::llm::LlmState>();
        let model_available = llm_state.store.is_available(&GEMMA4_E2B);
        let model_bundled = llm_state.store.is_bundled(&GEMMA4_E2B);
        let channel_guard = llm_state.channel.lock().await;
        let model_loaded = channel_guard
            .as_ref()
            .map(|ch| ch.is_alive())
            .unwrap_or(false);
        drop(channel_guard);

        let device = llm_state
            .device_name
            .get()
            .cloned()
            .unwrap_or_else(|| "cpu".to_string());

        Ok(LlmStatus {
            available: model_available,
            model_loaded,
            model_id: if model_available {
                Some(GEMMA4_E2B.id.to_string())
            } else {
                None
            },
            mode: if model_available { "llm" } else { "template" }.to_string(),
            device,
            bundled: model_bundled,
        })
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_available_models(
    app: tauri::AppHandle,
) -> Result<Vec<ModelStatus>, crate::error::AppError> {
    #[cfg(not(feature = "llm"))]
    {
        let _ = &app;
        Ok(vec![])
    }

    #[cfg(feature = "llm")]
    {
        use crate::llm::GEMMA4_E2B;

        let llm_state = app.state::<crate::llm::LlmState>();
        let sys_mem = get_system_memory_mb();
        let avail_mem = get_available_memory_mb();
        Ok(vec![ModelStatus {
            id: GEMMA4_E2B.id.to_string(),
            display_name: GEMMA4_E2B.display_name.to_string(),
            downloaded: llm_state.store.is_available(&GEMMA4_E2B),
            bundled: llm_state.store.is_bundled(&GEMMA4_E2B),
            file_size_mb: GEMMA4_E2B.file_size_mb,
            ram_requirement_mb: GEMMA4_E2B.ram_requirement_mb,
            system_memory_mb: sys_mem,
            available_memory_mb: avail_mem,
        }])
    }
}

#[tauri::command]
#[specta::specta]
pub async fn download_model(
    model_id: String,
    app: tauri::AppHandle,
) -> Result<(), crate::error::AppError> {
    #[cfg(not(feature = "llm"))]
    {
        let _ = (&model_id, &app);
        Err(crate::llm::LlmError::ModelNotFound("LLM feature not compiled".to_string()).into())
    }

    #[cfg(feature = "llm")]
    {
        use tauri::Emitter;

        let config = mentor_llm::download::get_config(&model_id)
            .ok_or(crate::llm::LlmError::ModelNotFound(model_id))?;

        let llm_state = app.state::<crate::llm::LlmState>();
        let store = llm_state.store.clone();
        let progress_handle = app.clone();

        // The download is blocking (hf-hub sync API); progress events are
        // throttled so the frontend isn't flooded.
        tokio::task::spawn_blocking(move || {
            let mut throttle = ProgressThrottle::new();
            store.download(config, |downloaded, total| {
                if throttle.should_emit(downloaded, total) {
                    let _ = progress_handle.emit(
                        "llm-download-progress",
                        serde_json::json!({
                            "downloadedBytes": downloaded,
                            "totalBytes": total,
                        }),
                    );
                }
            })
        })
        .await
        .map_err(|e| {
            crate::llm::LlmError::DownloadError(format!("download task panicked: {e}"))
        })??;

        Ok(())
    }
}

/// Throttles download progress emissions to every 256KB or 200ms.
///
/// Completion events (`downloaded == total`) are always emitted.
#[cfg(feature = "llm")]
struct ProgressThrottle {
    last_emitted_bytes: u64,
    last_emit: std::time::Instant,
}

#[cfg(feature = "llm")]
impl ProgressThrottle {
    const BYTE_THRESHOLD: u64 = 256 * 1024;
    const TIME_THRESHOLD: std::time::Duration = std::time::Duration::from_millis(200);

    fn new() -> Self {
        Self {
            last_emitted_bytes: 0,
            last_emit: std::time::Instant::now(),
        }
    }

    fn should_emit(&mut self, downloaded: u64, total: u64) -> bool {
        let done = total > 0 && downloaded >= total;
        if done
            || downloaded.saturating_sub(self.last_emitted_bytes) >= Self::BYTE_THRESHOLD
            || self.last_emit.elapsed() >= Self::TIME_THRESHOLD
        {
            self.last_emitted_bytes = downloaded;
            self.last_emit = std::time::Instant::now();
            true
        } else {
            false
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
#[specta::specta]
pub async fn generate_coaching(
    fen: String,
    classification: String,
    coaching_context: Option<crate::models::heuristics::CoachingContext>,
    player_move_san: String,
    engine_best_san: Option<String>,
    request_id: Option<String>,
    app: tauri::AppHandle,
    db: tauri::State<'_, std::sync::Mutex<crate::db::connection::Database>>,
    player_id: tauri::State<'_, crate::CurrentPlayerId>,
) -> Result<CoachingResponse, crate::error::AppError> {
    // Determine player level
    let level = determine_player_level(&db, &player_id)?;
    let level_str = match level {
        crate::llm::PlayerLevel::Beginner => "beginner",
        crate::llm::PlayerLevel::Intermediate => "intermediate",
        crate::llm::PlayerLevel::UpperIntermediate => "upperIntermediate",
    };

    // Extract themes from coaching context for cache key
    let themes = theme_names(coaching_context.as_ref());

    // Check cache
    let cache_key = crate::llm::cache::compute_cache_key(&fen, level_str, &classification, &themes);
    {
        let db_lock = db
            .lock()
            .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
        if let Some((text, _)) = db_lock.get_cached_coaching(&cache_key)? {
            return Ok(CoachingResponse {
                text,
                source: CoachingSource::Cache,
            });
        }
    }

    // Try LLM generation (feature-gated)
    #[cfg(feature = "llm")]
    {
        let llm_state = app.state::<crate::llm::LlmState>();
        if llm_state.store.is_available(&crate::llm::GEMMA4_E2B) {
            let params = LlmCoachingParams {
                level,
                fen: &fen,
                classification: &classification,
                coaching_context: coaching_context.as_ref(),
                player_move_san: &player_move_san,
                engine_best_san: engine_best_san.as_deref(),
            };
            match tokio::time::timeout(
                std::time::Duration::from_secs(30),
                try_llm_generation(&llm_state, &app, request_id.as_deref(), &params),
            )
            .await
            {
                Err(_) => {
                    tracing::warn!("LLM generation timed out after 30s, falling back to template");
                    emit_llm_error(&app, request_id.as_deref(), "Generation timed out");
                }
                Ok(inner) => match inner {
                    Ok(text) => {
                        // Cache the LLM result
                        if let Ok(db_lock) = db
                            .lock()
                            .map_err(|e| crate::error::AppError::Lock(e.to_string()))
                        {
                            let _ = db_lock.set_cached_coaching(
                                &cache_key,
                                &text,
                                level_str,
                                &classification,
                                &fen,
                                30,
                            );
                        }
                        return Ok(CoachingResponse {
                            text,
                            source: CoachingSource::Llm,
                        });
                    }
                    Err(e) => {
                        tracing::warn!("LLM generation failed, falling back to template: {e}");
                        emit_llm_error(&app, request_id.as_deref(), &e.to_string());
                    }
                },
            }
        }
    }

    #[cfg(not(feature = "llm"))]
    let _ = (&app, &request_id, &player_move_san, &engine_best_san);

    // Fall back to template
    let template_text = generate_template_fallback(&classification, coaching_context.as_ref());

    Ok(CoachingResponse {
        text: template_text,
        source: CoachingSource::Template,
    })
}

/// Parameters for LLM coaching generation, grouped to keep the function signature clean.
#[cfg(feature = "llm")]
struct LlmCoachingParams<'a> {
    level: crate::llm::PlayerLevel,
    fen: &'a str,
    classification: &'a str,
    coaching_context: Option<&'a CoachingContext>,
    player_move_san: &'a str,
    engine_best_san: Option<&'a str>,
}

/// Attempt LLM-based coaching text generation.
#[cfg(feature = "llm")]
async fn try_llm_generation(
    llm_state: &crate::llm::LlmState,
    app: &tauri::AppHandle,
    request_id: Option<&str>,
    params: &LlmCoachingParams<'_>,
) -> Result<String, crate::llm::LlmError> {
    use crate::llm::LlmTokenEvent;
    use crate::llm::position_facts::{EngineData, MoveInput, build_move_facts};
    use tauri::Emitter;

    // Engine data arrives with the full plumbing (CoachingEngineData); until
    // then the best-move SAN is the only engine fact available here.
    let engine_data = params.engine_best_san.map(|san| EngineData {
        best_move_san: Some(san.to_string()),
        ..EngineData::default()
    });

    let facts = build_move_facts(
        &MoveInput {
            fen_before: params.fen,
            player_move_san: params.player_move_san,
            player_move_uci: None,
            classification: params.classification,
        },
        params.coaching_context,
        engine_data.as_ref(),
    );
    let prompt = crate::llm::coach_prompt::build_coaching_prompt(params.level, &facts);

    // Spawn the inference channel if not yet created (normally warmed at startup)
    llm_state.ensure_channel().await?;
    let submit_result = {
        let mut channel_guard = llm_state.channel.lock().await;
        let channel = channel_guard
            .as_mut()
            .ok_or(crate::llm::LlmError::ModelNotLoaded)?;
        channel.submit(prompt).await?
        // channel_guard dropped here — BEFORE we await the result.
    };

    // Spawn a task to forward token events to the frontend
    if let Some(rid) = request_id {
        let app_handle = app.clone();
        let rid = rid.to_string();
        let mut token_rx = submit_result.token_rx;
        tokio::spawn(async move {
            while let Some(text) = token_rx.recv().await {
                let _ = app_handle.emit(
                    "llm-token",
                    LlmTokenEvent::Token {
                        text,
                        request_id: rid.clone(),
                    },
                );
            }
        });
    }

    // Await the response outside the mutex lock
    let result = submit_result
        .response_rx
        .await
        .map_err(|_| crate::llm::LlmError::InferenceError("Channel closed".to_string()))??;

    // Emit Done event
    if let Some(rid) = request_id {
        let _ = app.emit(
            "llm-token",
            LlmTokenEvent::Done {
                full_text: result.clone(),
                request_id: rid.to_string(),
            },
        );
    }

    Ok(result)
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
#[specta::specta]
pub async fn generate_game_summary(
    result: String,
    outcome_type: String,
    move_count: usize,
    accuracy_pct: f64,
    best_moves: usize,
    blunders: usize,
    mistakes: usize,
    inaccuracies: usize,
    app: tauri::AppHandle,
) -> Result<String, crate::error::AppError> {
    let prompt = crate::llm::coach_prompt::build_game_summary_prompt(
        &result,
        &outcome_type,
        move_count,
        accuracy_pct,
        best_moves,
        blunders,
        mistakes,
        inaccuracies,
    );

    // Try LLM with 15s timeout, fallback to template
    match tokio::time::timeout(
        std::time::Duration::from_secs(15),
        try_generate_summary(&app, &prompt),
    )
    .await
    {
        Ok(Ok(text)) if !text.trim().is_empty() => Ok(text.trim().to_string()),
        _ => {
            let template = if blunders == 0 && mistakes == 0 {
                "Solid game with no major errors.".to_string()
            } else if blunders > 2 {
                "A challenging game — focus on avoiding blunders in critical moments.".to_string()
            } else {
                format!("A {}-move game with room to improve accuracy.", move_count)
            };
            Ok(template)
        }
    }
}

/// Attempt LLM-based game summary generation.
#[cfg(feature = "llm")]
async fn try_generate_summary(
    app: &tauri::AppHandle,
    prompt: &str,
) -> Result<String, crate::llm::LlmError> {
    use crate::llm::GEMMA4_E2B;

    let llm_state = app.state::<crate::llm::LlmState>();
    if !llm_state.store.is_available(&GEMMA4_E2B) {
        return Err(crate::llm::LlmError::ModelNotFound(
            "Model not available".to_string(),
        ));
    }

    llm_state.ensure_channel().await?;
    let submit_result = {
        let mut channel_guard = llm_state.channel.lock().await;
        let channel = channel_guard
            .as_mut()
            .ok_or(crate::llm::LlmError::ModelNotLoaded)?;
        channel.submit(prompt.to_string()).await?
    };

    // Drain token stream (we only need the final result)
    let mut token_rx = submit_result.token_rx;
    tokio::spawn(async move { while token_rx.recv().await.is_some() {} });

    let result = submit_result
        .response_rx
        .await
        .map_err(|_| crate::llm::LlmError::InferenceError("Channel closed".to_string()))??;

    Ok(result)
}

/// No-op fallback when LLM feature is disabled.
#[cfg(not(feature = "llm"))]
async fn try_generate_summary(
    _app: &tauri::AppHandle,
    _prompt: &str,
) -> Result<String, crate::llm::LlmError> {
    Err(crate::llm::LlmError::ModelNotFound(
        "LLM feature not compiled".to_string(),
    ))
}

fn determine_player_level(
    db: &tauri::State<'_, std::sync::Mutex<crate::db::connection::Database>>,
    player_id: &tauri::State<'_, crate::CurrentPlayerId>,
) -> Result<crate::llm::PlayerLevel, crate::error::AppError> {
    let pid = player_id.get()?;
    if pid.is_empty() {
        return Ok(crate::llm::PlayerLevel::Beginner);
    }

    let db_lock = db
        .lock()
        .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;

    // Check for coaching level override in player settings
    let player = db_lock.get_or_create_player("Player")?;
    if let Some(ref override_level) = player.settings.coaching_level {
        return Ok(match override_level.as_str() {
            "beginner" => crate::llm::PlayerLevel::Beginner,
            "intermediate" => crate::llm::PlayerLevel::Intermediate,
            "upperIntermediate" => crate::llm::PlayerLevel::UpperIntermediate,
            _ => crate::llm::PlayerLevel::Beginner,
        });
    }

    // Derive from game stats
    let games = player.games_played;
    Ok(crate::llm::PlayerLevel::from_game_stats(games, 0.10, 0.10))
}

fn generate_template_fallback(
    classification: &str,
    coaching_context: Option<&CoachingContext>,
) -> String {
    let mc = crate::models::engine::MoveClassification::from_str_loose(classification);

    match coaching_context {
        Some(ctx) => crate::coaching::generate_coaching_text(&mc, ctx),
        None => crate::coaching::templates::generic_template(mc).to_string(),
    }
}

/// Emit an LLM error event to the frontend so it can handle the failure gracefully.
#[cfg(feature = "llm")]
fn emit_llm_error(app: &tauri::AppHandle, request_id: Option<&str>, message: &str) {
    use tauri::Emitter;

    if let Some(rid) = request_id {
        let _ = app.emit(
            "llm-token",
            crate::llm::LlmTokenEvent::Error {
                message: message.to_string(),
                request_id: rid.to_string(),
            },
        );
    }
}
