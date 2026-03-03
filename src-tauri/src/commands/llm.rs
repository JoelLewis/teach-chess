use serde::Serialize;

#[cfg(feature = "llm")]
use tauri::Manager;

use crate::llm::{CoachingResponse, CoachingSource};

/// Extract a string array from a JSON object at the given key.
fn extract_string_array(ctx: &Option<serde_json::Value>, key: &str) -> Vec<String> {
    ctx.as_ref()
        .and_then(|c| c.get(key))
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Status of the LLM subsystem
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmStatus {
    pub available: bool,
    pub model_loaded: bool,
    pub model_id: Option<String>,
    pub mode: String,
}

/// Download/availability status of a single model
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub id: String,
    pub display_name: String,
    pub downloaded: bool,
    pub file_size_mb: u32,
    pub ram_requirement_mb: u32,
    pub system_memory_mb: u32,
}

/// Detect total system memory in MB.
#[allow(dead_code)]
fn get_system_memory_mb() -> u32 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(contents) = std::fs::read_to_string("/proc/meminfo") {
            for line in contents.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(kb_str) = parts.get(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            return (kb / 1024) as u32;
                        }
                    }
                }
            }
        }
        0
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("sysctl").arg("-n").arg("hw.memsize").output() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                if let Ok(bytes) = s.trim().parse::<u64>() {
                    return (bytes / (1024 * 1024)) as u32;
                }
            }
        }
        0
    }

    #[cfg(target_os = "windows")]
    {
        // Approximate via GlobalMemoryStatusEx
        0 // Would need winapi crate; skip for now
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        0
    }
}

#[tauri::command]
pub async fn get_llm_status(app: tauri::AppHandle) -> Result<LlmStatus, crate::error::AppError> {
    #[cfg(not(feature = "llm"))]
    {
        let _ = &app;
        return Ok(LlmStatus {
            available: false,
            model_loaded: false,
            model_id: None,
            mode: "template".to_string(),
        });
    }

    #[cfg(feature = "llm")]
    {
        use crate::llm::model_manager::GEMMA2_2B;

        let llm_state = app.state::<crate::llm::LlmState>();
        let model_available = llm_state.model_manager.is_available(&GEMMA2_2B);
        let channel_guard = llm_state.channel.lock().await;
        let model_loaded = channel_guard
            .as_ref()
            .map(|ch| ch.is_alive())
            .unwrap_or(false);
        drop(channel_guard);

        Ok(LlmStatus {
            available: model_available,
            model_loaded,
            model_id: if model_available {
                Some(GEMMA2_2B.id.to_string())
            } else {
                None
            },
            mode: if model_available { "llm" } else { "template" }.to_string(),
        })
    }
}

#[tauri::command]
pub async fn get_available_models(
    app: tauri::AppHandle,
) -> Result<Vec<ModelStatus>, crate::error::AppError> {
    #[cfg(not(feature = "llm"))]
    {
        let _ = &app;
        return Ok(vec![]);
    }

    #[cfg(feature = "llm")]
    {
        use crate::llm::model_manager::GEMMA2_2B;

        let llm_state = app.state::<crate::llm::LlmState>();
        let sys_mem = get_system_memory_mb();
        Ok(vec![ModelStatus {
            id: GEMMA2_2B.id.to_string(),
            display_name: GEMMA2_2B.display_name.to_string(),
            downloaded: llm_state.model_manager.is_available(&GEMMA2_2B),
            file_size_mb: GEMMA2_2B.file_size_mb,
            ram_requirement_mb: GEMMA2_2B.ram_requirement_mb,
            system_memory_mb: sys_mem,
        }])
    }
}

#[tauri::command]
pub async fn download_model(
    model_id: String,
    app: tauri::AppHandle,
) -> Result<(), crate::error::AppError> {
    #[cfg(not(feature = "llm"))]
    {
        let _ = (&model_id, &app);
        return Err(crate::llm::LlmError::ModelNotFound(
            "LLM feature not compiled".to_string(),
        )
        .into());
    }

    #[cfg(feature = "llm")]
    {
        use crate::llm::model_manager::ModelManager;

        let config = ModelManager::get_config(&model_id)
            .ok_or_else(|| crate::llm::LlmError::ModelNotFound(model_id))?;

        let llm_state = app.state::<crate::llm::LlmState>();
        llm_state.model_manager.download(config, &app).await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn generate_coaching(
    fen: String,
    classification: String,
    coaching_context: Option<serde_json::Value>,
    player_move_san: String,
    engine_best_san: Option<String>,
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
    let themes = extract_string_array(&coaching_context, "themes");

    // Check cache
    let cache_key =
        crate::llm::cache::compute_cache_key(&fen, level_str, &classification, &themes);
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
        if llm_state
            .model_manager
            .is_available(&crate::llm::model_manager::GEMMA2_2B)
        {
            match try_llm_generation(
                &llm_state,
                &level,
                &classification,
                &coaching_context,
                &player_move_san,
                &engine_best_san,
            )
            .await
            {
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
                }
            }
        }
    }

    #[cfg(not(feature = "llm"))]
    let _ = &app;

    // Fall back to template
    let template_text = generate_template_fallback(
        &classification,
        &coaching_context,
        &player_move_san,
        &engine_best_san,
    );

    Ok(CoachingResponse {
        text: template_text,
        source: CoachingSource::Template,
    })
}

/// Attempt LLM-based coaching text generation.
#[cfg(feature = "llm")]
async fn try_llm_generation(
    llm_state: &crate::llm::LlmState,
    level: &crate::llm::PlayerLevel,
    classification: &str,
    coaching_context: &Option<serde_json::Value>,
    player_move_san: &str,
    engine_best_san: &Option<String>,
) -> Result<String, crate::llm::LlmError> {
    use crate::llm::model_manager::GEMMA2_2B;

    // Extract fields for the prompt
    let phase = coaching_context
        .as_ref()
        .and_then(|ctx| ctx.get("phase"))
        .and_then(|p| p.as_str())
        .unwrap_or("middlegame");

    let themes = extract_string_array(coaching_context, "themes");

    let tactics: Vec<String> = coaching_context
        .as_ref()
        .and_then(|ctx| ctx.get("tactics"))
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    v.as_object()
                        .and_then(|o| o.get("tacticType"))
                        .and_then(|tt| tt.as_str())
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default();

    let material_balance = coaching_context
        .as_ref()
        .and_then(|ctx| ctx.get("material"))
        .and_then(|m| m.get("balanceCp"))
        .and_then(|b| b.as_i64())
        .unwrap_or(0) as i32;

    let prompt = crate::llm::prompts::build_prompt(
        level,
        classification,
        phase,
        player_move_san,
        engine_best_san.as_deref(),
        &themes,
        &tactics,
        material_balance,
    );

    // Lazy-spawn the inference channel if not yet created
    let response_rx = {
        let mut channel_guard = llm_state.channel.lock().await;
        if channel_guard.is_none() {
            let model_path = llm_state.model_manager.get_model_path(&GEMMA2_2B);
            let tokenizer_path = llm_state.model_manager.get_tokenizer_path(&GEMMA2_2B);
            let ch = crate::llm::channel::InferenceChannel::spawn(&model_path, &tokenizer_path)?;
            *channel_guard = Some(ch);
        }

        let channel = channel_guard.as_mut().unwrap();
        channel.submit(prompt).await?
        // channel_guard dropped here at end of block — BEFORE we await the result.
        // This is critical: inference takes 2-5s and holding the lock would block
        // all other coaching calls during that time.
    };

    // Await the response outside the mutex lock
    let result = response_rx
        .await
        .map_err(|_| crate::llm::LlmError::InferenceError("Channel closed".to_string()))??;

    Ok(result)
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
    Ok(crate::llm::PlayerLevel::from_game_stats(
        games, 0.10, 0.10,
    ))
}

fn generate_template_fallback(
    classification: &str,
    coaching_context: &Option<serde_json::Value>,
    _player_move_san: &str,
    _engine_best_san: &Option<String>,
) -> String {
    let mc = crate::models::engine::MoveClassification::from_str_loose(classification);

    if let Some(ctx_value) = coaching_context {
        if let Ok(ctx) = serde_json::from_value::<crate::models::heuristics::CoachingContext>(
            ctx_value.clone(),
        ) {
            return crate::coaching::generate_coaching_text(&mc, &ctx);
        }
    }

    crate::coaching::templates::generic_template(mc).to_string()
}
