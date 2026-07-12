#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use tracing::{debug, info, warn};

use crate::error::LlmError;
use crate::stream::ChannelFilter;

/// Inference parameters for coaching text generation.
const TEMPERATURE: f32 = 0.3;
const TOP_K: i32 = 50;
const TOP_P: f32 = 0.9;
const REPEAT_PENALTY_LAST_N: i32 = 64;
const REPEAT_PENALTY: f32 = 1.1;
const SEED: u32 = 42;
const N_CTX: u32 = 1024;

/// Number of GPU layers to offload plus a display name for the device.
///
/// macOS builds always have Metal compiled in and use it by default;
/// `CHESS_MENTOR_DEVICE=cpu` forces CPU inference everywhere.
fn gpu_config() -> (u32, &'static str) {
    if matches!(std::env::var("CHESS_MENTOR_DEVICE").as_deref(), Ok("cpu")) {
        info!("CHESS_MENTOR_DEVICE=cpu — forcing CPU inference");
        return (0, "cpu");
    }
    compiled_gpu_config()
}

#[cfg(any(target_os = "macos", feature = "llm-metal"))]
fn compiled_gpu_config() -> (u32, &'static str) {
    (u32::MAX, "metal")
}

#[cfg(all(
    not(any(target_os = "macos", feature = "llm-metal")),
    feature = "llm-cuda"
))]
fn compiled_gpu_config() -> (u32, &'static str) {
    (u32::MAX, "cuda")
}

#[cfg(not(any(target_os = "macos", feature = "llm-metal", feature = "llm-cuda")))]
fn compiled_gpu_config() -> (u32, &'static str) {
    (0, "cpu")
}

/// Display name for the compute device inference will use.
pub fn device_name() -> &'static str {
    gpu_config().1
}

/// Manages a loaded LLM model. `LlamaModel` is `Send+Sync`, so this can live in an `Arc`.
/// `LlamaContext` is `!Send`, so each `generate()` call creates a fresh context.
/// Clone is cheap — both fields are `Arc`.
#[derive(Clone, Debug)]
pub struct ModelManager {
    backend: Arc<LlamaBackend>,
    model: Arc<LlamaModel>,
}

// LlamaBackend and LlamaModel are Send+Sync in llama-cpp-2
unsafe impl Send for ModelManager {}
unsafe impl Sync for ModelManager {}

impl ModelManager {
    /// Load a GGUF model from disk. Blocking — call from `spawn_blocking`.
    pub fn load(model_path: &Path) -> Result<Self, LlmError> {
        if !model_path.exists() {
            return Err(LlmError::ModelNotFound(model_path.display().to_string()));
        }

        let (n_gpu_layers, device) = gpu_config();
        info!("Loading LLM model from {} ({device})", model_path.display());

        let backend = LlamaBackend::init()
            .map_err(|e| LlmError::InferenceError(format!("backend init: {e}")))?;

        let model_params = LlamaModelParams::default().with_n_gpu_layers(n_gpu_layers);
        let model = LlamaModel::load_from_file(&backend, model_path, &model_params)
            .map_err(|e| LlmError::InferenceError(format!("model load: {e}")))?;

        info!(
            "LLM model loaded successfully ({} params)",
            model.n_params()
        );

        Ok(Self {
            backend: Arc::new(backend),
            model: Arc::new(model),
        })
    }

    /// Generate text from a formatted prompt. Blocking — call from `spawn_blocking`.
    pub fn generate(&self, prompt: &str, max_tokens: u32) -> Result<String, LlmError> {
        self.generate_streaming(prompt, max_tokens, |_| {})
    }

    /// Generate text with per-token streaming callback. Blocking.
    pub fn generate_streaming(
        &self,
        prompt: &str,
        max_tokens: u32,
        on_token: impl FnMut(&str),
    ) -> Result<String, LlmError> {
        self.generate_cancellable(prompt, max_tokens, on_token, || false)
    }

    /// Generate text with streaming callback and cooperative cancellation. Blocking.
    ///
    /// `is_cancelled` is polled between tokens; returns [`LlmError::Cancelled`]
    /// when it reports true. Creates a fresh `LlamaContext` for thread safety.
    pub fn generate_cancellable(
        &self,
        prompt: &str,
        max_tokens: u32,
        mut on_token: impl FnMut(&str),
        is_cancelled: impl Fn() -> bool,
    ) -> Result<String, LlmError> {
        let ctx_params =
            LlamaContextParams::default().with_n_ctx(Some(NonZeroU32::new(N_CTX).unwrap()));

        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| LlmError::InferenceError(format!("context creation: {e}")))?;

        // Tokenize prompt
        let tokens = self
            .model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| LlmError::InferenceError(format!("tokenization: {e}")))?;

        debug!("Prompt tokenized: {} tokens", tokens.len());

        if tokens.is_empty() {
            return Err(LlmError::InferenceError("empty prompt".to_string()));
        }
        if tokens.len() >= N_CTX as usize {
            return Err(LlmError::InferenceError(
                "prompt exceeds context window".to_string(),
            ));
        }

        // Process prompt tokens in a batch
        let mut batch = LlamaBatch::new(N_CTX as usize, 1);
        let last_idx = (tokens.len() - 1) as i32;
        for (i, token) in tokens.into_iter().enumerate() {
            let pos = i as i32;
            batch
                .add(token, pos, &[0], pos == last_idx)
                .map_err(|e| LlmError::InferenceError(format!("batch add: {e}")))?;
        }

        ctx.decode(&mut batch)
            .map_err(|e| LlmError::InferenceError(format!("prompt decode: {e}")))?;

        // Sampler chain: temp → top_k → top_p → penalties → dist
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(TEMPERATURE),
            LlamaSampler::top_k(TOP_K),
            LlamaSampler::top_p(TOP_P, 1),
            LlamaSampler::penalties(REPEAT_PENALTY_LAST_N, REPEAT_PENALTY, 0.0, 0.0),
            LlamaSampler::dist(SEED),
        ]);

        let mut output = String::new();
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut filter = ChannelFilter::new();
        let n_start = batch.n_tokens();

        for n_cur in n_start..n_start + max_tokens as i32 {
            if is_cancelled() {
                return Err(LlmError::Cancelled);
            }

            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(token);

            if self.model.is_eog_token(token) {
                debug!("End of generation token reached");
                break;
            }

            let piece = self
                .model
                .token_to_piece(token, &mut decoder, true, None)
                .map_err(|e| LlmError::InferenceError(format!("token decode: {e}")))?;

            let visible = filter.push(&piece);
            if !visible.is_empty() {
                on_token(&visible);
                output.push_str(&visible);
            }

            batch.clear();
            batch
                .add(token, n_cur, &[0], true)
                .map_err(|e| LlmError::InferenceError(format!("batch add gen: {e}")))?;

            ctx.decode(&mut batch)
                .map_err(|e| LlmError::InferenceError(format!("decode gen: {e}")))?;
        }

        let tail = filter.finish();
        if !tail.is_empty() {
            on_token(&tail);
            output.push_str(&tail);
        }

        if output.is_empty() {
            warn!("LLM generated empty output");
        }

        Ok(output.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn load_missing_file_returns_error() {
        let result = ModelManager::load(&PathBuf::from("/nonexistent/model.gguf"));
        assert!(matches!(result, Err(LlmError::ModelNotFound(_))));
    }

    #[test]
    fn device_name_is_known_value() {
        assert!(matches!(device_name(), "cpu" | "metal" | "cuda"));
    }

    /// End-to-end inference smoke test against the real GGUF.
    ///
    /// Requires the model in src-tauri/models/ (run scripts/fetch-model.sh first).
    /// Run with: cargo test -p mentor-llm --features llm --release -- --ignored llm_generates_coherent_text --nocapture
    #[test]
    #[ignore]
    fn llm_generates_coherent_text() {
        use crate::download::GEMMA4_E2B;
        use crate::prompts::format_chat;

        let model_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../src-tauri/models")
            .join(GEMMA4_E2B.gguf_filename);
        assert!(
            model_path.exists(),
            "model file missing — run scripts/fetch-model.sh first"
        );

        eprintln!("inference device: {}", device_name());
        let t0 = std::time::Instant::now();
        let manager = ModelManager::load(&model_path).expect("model should load");
        eprintln!("load took {:.1}s", t0.elapsed().as_secs_f32());

        let prompt = format_chat(
            "You are a concise chess coach.",
            "Reply with one short sentence: why is controlling the center important in chess?",
        );
        let mut streamed = String::new();
        let mut chunks = 0u32;
        let t1 = std::time::Instant::now();
        let text = manager
            .generate_streaming(&prompt, 128, |t| {
                chunks += 1;
                streamed.push_str(t);
            })
            .expect("generation should succeed");
        let gen_secs = t1.elapsed().as_secs_f32();
        eprintln!(
            "generation took {gen_secs:.1}s for {chunks} chunks ({:.2} chunks/s)",
            chunks as f32 / gen_secs
        );
        eprintln!("output: {text}");

        assert!(text.len() > 20, "suspiciously short output: {text:?}");
        assert!(
            !text.contains("<|turn>") && !text.contains("<turn|>"),
            "output leaked turn markers: {text:?}"
        );
        assert!(
            !text.contains("<|channel>") && !text.contains("<channel|>"),
            "output leaked channel markers: {text:?}"
        );
        assert_eq!(
            streamed.trim(),
            text,
            "streamed text should match final text"
        );
    }
}
