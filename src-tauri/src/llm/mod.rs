pub mod cache;

#[cfg(feature = "llm")]
pub mod channel;

use serde::{Deserialize, Serialize};

pub use mentor_llm::prompts;
pub use mentor_llm::{LlmError, PlayerLevel};

#[cfg(feature = "llm")]
pub use mentor_llm::download::GEMMA4_E2B;

#[cfg(feature = "llm")]
use std::path::PathBuf;

/// Managed state for the LLM subsystem. Lazily initializes the inference channel.
#[cfg(feature = "llm")]
pub struct LlmState {
    pub store: mentor_llm::download::ModelStore,
    /// Lazy-initialized: None until the first coaching request with a downloaded model.
    pub channel: tokio::sync::Mutex<Option<channel::InferenceChannel>>,
    /// Which compute device the inference channel is using (e.g. "cpu", "cuda", "metal").
    pub device_name: std::sync::OnceLock<String>,
}

#[cfg(feature = "llm")]
impl LlmState {
    pub fn new(app_data_dir: PathBuf, resource_dir: Option<PathBuf>) -> Self {
        let store = mentor_llm::download::ModelStore::new(&app_data_dir, resource_dir);
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
            let model_path = self.store.model_path(&GEMMA4_E2B);
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
