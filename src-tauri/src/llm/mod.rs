pub mod cache;
pub mod player_level;
pub mod prompts;

#[cfg(feature = "llm")]
pub mod candle_backend;
#[cfg(feature = "llm")]
pub mod channel;
#[cfg(feature = "llm")]
pub mod model_manager;

use serde::{Deserialize, Serialize};

#[cfg(feature = "llm")]
use std::path::PathBuf;

/// Managed state for the LLM subsystem. Lazily initializes the inference channel.
#[cfg(feature = "llm")]
#[allow(dead_code)]
pub struct LlmState {
    pub model_manager: model_manager::ModelManager,
    /// Lazy-initialized: None until the first coaching request with a downloaded model.
    pub channel: tokio::sync::Mutex<Option<channel::InferenceChannel>>,
    pub app_data_dir: PathBuf,
}

#[cfg(feature = "llm")]
impl LlmState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let model_manager = model_manager::ModelManager::new(&app_data_dir);
        Self {
            model_manager,
            channel: tokio::sync::Mutex::new(None),
            app_data_dir,
        }
    }
}

/// Player skill level for coaching tone adaptation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlayerLevel {
    Beginner,
    Intermediate,
    UpperIntermediate,
}

/// Errors from the LLM subsystem
#[derive(Debug, Clone, thiserror::Error)]
#[allow(dead_code)]
pub enum LlmError {
    #[error("Model not loaded")]
    ModelNotLoaded,

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Inference error: {0}")]
    InferenceError(String),

    #[error("Download error: {0}")]
    DownloadError(String),

    #[error("Inference cancelled")]
    Cancelled,

    #[error("Tokenizer error: {0}")]
    TokenizerError(String),
}

/// Source of a coaching response
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoachingSource {
    Cache,
    Llm,
    Template,
}

/// A coaching request from the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CoachingRequest {
    pub fen: String,
    pub classification: String,
    pub context: serde_json::Value,
    pub player_level: PlayerLevel,
    pub player_move_san: String,
    pub engine_best_san: Option<String>,
}

/// A coaching response returned to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoachingResponse {
    pub text: String,
    pub source: CoachingSource,
}
