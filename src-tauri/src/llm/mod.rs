pub mod cache;
pub mod coach_prompt;
pub mod player_level;
pub mod position_facts;

#[cfg(feature = "llm")]
pub mod channel;
#[cfg(feature = "llm")]
pub mod llm_support;

use serde::{Deserialize, Serialize};

pub use player_level::PlayerLevel;
pub use sensei_llm::LlmError;

#[cfg(feature = "llm")]
pub use llm_support::GEMMA4_E2B;

#[cfg(feature = "llm")]
use std::path::PathBuf;

/// Managed state for the LLM subsystem. Lazily initializes the inference channel.
#[cfg(feature = "llm")]
pub struct LlmState {
    pub store: sensei_llm::ModelStore,
    /// Lazy-initialized: None until the first coaching request with a downloaded model.
    pub channel: tokio::sync::Mutex<Option<channel::InferenceChannel>>,
    /// Which compute device the inference channel is using (e.g. "cpu", "cuda", "metal").
    pub device_name: std::sync::OnceLock<String>,
}

#[cfg(feature = "llm")]
impl LlmState {
    pub fn new(app_data_dir: PathBuf, resource_dir: Option<PathBuf>) -> Self {
        let store = llm_support::model_store(&app_data_dir, resource_dir);
        Self {
            store,
            channel: tokio::sync::Mutex::new(None),
            device_name: std::sync::OnceLock::new(),
        }
    }

    /// Initialize the inference channel if it hasn't been created yet.
    ///
    /// The channel's worker loads the model on a blocking thread, so this
    /// returns quickly; generation requests queue until the load completes.
    /// Once created the channel is never reset to `None`.
    pub async fn ensure_channel(&self) -> Result<(), LlmError> {
        let mut channel_guard = self.channel.lock().await;
        if channel_guard.is_none() {
            let model_path = self.store.model_path(GEMMA4_E2B.spec);
            let (ch, dev_name) = channel::InferenceChannel::spawn(&model_path);
            let _ = self.device_name.set(dev_name.clone());
            *channel_guard = Some(ch);
            tracing::info!("Inference channel initialized on {dev_name}");
        }
        Ok(())
    }
}

/// Source of a coaching response
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum CoachingSource {
    Cache,
    Llm,
    Template,
}

/// A coaching response returned to the frontend
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CoachingResponse {
    pub text: String,
    pub source: CoachingSource,
}

/// Events emitted during streaming LLM token generation.
#[cfg(feature = "llm")]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum LlmTokenEvent {
    Token {
        text: String,
        request_id: String,
    },
    Done {
        full_text: String,
        request_id: String,
    },
    Error {
        message: String,
        request_id: String,
    },
}
