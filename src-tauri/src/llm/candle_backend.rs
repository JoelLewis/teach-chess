#![allow(dead_code)]

use std::path::Path;

use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_gemma3::ModelWeights;
use tokenizers::Tokenizer;
use tokio_util::sync::CancellationToken;

use super::LlmError;

/// Candle-based inference backend for quantized Gemma 2B GGUF models.
pub struct CandleBackend {
    model: Option<ModelWeights>,
    tokenizer: Option<Tokenizer>,
    device: Device,
}

impl CandleBackend {
    pub fn new() -> Self {
        Self {
            model: None,
            tokenizer: None,
            device: Device::Cpu,
        }
    }

    /// Load a quantized GGUF model and tokenizer from disk.
    pub fn load(&mut self, model_path: &Path, tokenizer_path: &Path) -> Result<(), LlmError> {
        // Load GGUF model
        let mut model_file = std::fs::File::open(model_path)
            .map_err(|e| LlmError::InferenceError(format!("Failed to open model: {e}")))?;

        let content = candle_core::quantized::gguf_file::Content::read(&mut model_file)
            .map_err(|e| LlmError::InferenceError(format!("Failed to read GGUF: {e}")))?;

        let model = ModelWeights::from_gguf(content, &mut model_file, &self.device)
            .map_err(|e| LlmError::InferenceError(format!("Failed to load model weights: {e}")))?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| LlmError::TokenizerError(format!("Failed to load tokenizer: {e}")))?;

        self.model = Some(model);
        self.tokenizer = Some(tokenizer);

        tracing::info!("Candle backend loaded model from {}", model_path.display());
        Ok(())
    }

    /// Whether a model is currently loaded.
    pub fn is_loaded(&self) -> bool {
        self.model.is_some() && self.tokenizer.is_some()
    }

    /// Unload model and tokenizer to free memory.
    pub fn unload(&mut self) {
        self.model = None;
        self.tokenizer = None;
        tracing::info!("Candle backend unloaded");
    }

    /// Generate text from a prompt using token-by-token decoding.
    ///
    /// Supports cancellation via the provided token.
    /// Uses temperature 0.3, top_p 0.9, repeat_penalty 1.1, max 128 tokens.
    pub fn generate(
        &mut self,
        prompt: &str,
        cancel: &CancellationToken,
    ) -> Result<String, LlmError> {
        let model = self.model.as_mut().ok_or(LlmError::ModelNotLoaded)?;
        let tokenizer = self.tokenizer.as_ref().ok_or(LlmError::ModelNotLoaded)?;

        // Tokenize prompt
        let encoding = tokenizer
            .encode(prompt, true)
            .map_err(|e| LlmError::TokenizerError(e.to_string()))?;
        let prompt_tokens = encoding.get_ids().to_vec();

        if prompt_tokens.is_empty() {
            return Err(LlmError::InferenceError("Empty prompt".to_string()));
        }

        let mut logits_processor = LogitsProcessor::from_sampling(
            42, // seed
            candle_transformers::generation::Sampling::TopKThenTopP {
                k: 50,
                p: 0.9,
                temperature: 0.3,
            },
        );

        let eos_token = tokenizer
            .token_to_id("<eos>")
            .or_else(|| tokenizer.token_to_id("</s>"))
            .or_else(|| tokenizer.token_to_id("<end_of_turn>"))
            .unwrap_or(1); // Fallback EOS token ID

        let max_tokens: usize = 128;
        let mut output_tokens: Vec<u32> = Vec::with_capacity(max_tokens);
        let mut all_tokens = prompt_tokens.clone();

        // Process prompt tokens as a batch first
        let input = Tensor::new(prompt_tokens.as_slice(), &self.device)
            .map_err(|e| LlmError::InferenceError(format!("Tensor creation: {e}")))?
            .unsqueeze(0)
            .map_err(|e| LlmError::InferenceError(format!("Unsqueeze: {e}")))?;

        let logits = model
            .forward(&input, 0)
            .map_err(|e| LlmError::InferenceError(format!("Forward pass: {e}")))?;

        // Get logits for the last position
        let logits = logits
            .squeeze(0)
            .map_err(|e| LlmError::InferenceError(format!("Squeeze: {e}")))?;
        let logits = logits
            .get(
                logits
                    .dim(0)
                    .map_err(|e| LlmError::InferenceError(e.to_string()))?
                    - 1,
            )
            .map_err(|e| LlmError::InferenceError(format!("Get last: {e}")))?;

        // Apply repeat penalty
        let logits = apply_repeat_penalty(&logits, 1.1, &all_tokens)?;

        let next_token = logits_processor
            .sample(&logits)
            .map_err(|e| LlmError::InferenceError(format!("Sampling: {e}")))?;

        if next_token == eos_token {
            return Ok(String::new());
        }

        output_tokens.push(next_token);
        all_tokens.push(next_token);

        // Generate remaining tokens one at a time
        for i in 0..max_tokens - 1 {
            if cancel.is_cancelled() {
                return Err(LlmError::Cancelled);
            }

            let input = Tensor::new(&[next_token], &self.device)
                .map_err(|e| LlmError::InferenceError(format!("Tensor: {e}")))?
                .unsqueeze(0)
                .map_err(|e| LlmError::InferenceError(format!("Unsqueeze: {e}")))?;

            let pos = prompt_tokens.len() + i + 1;
            let logits = model
                .forward(&input, pos)
                .map_err(|e| LlmError::InferenceError(format!("Forward: {e}")))?;

            let logits = logits
                .squeeze(0)
                .map_err(|e| LlmError::InferenceError(format!("Squeeze: {e}")))?
                .get(0)
                .map_err(|e| LlmError::InferenceError(format!("Get: {e}")))?;

            let logits = apply_repeat_penalty(&logits, 1.1, &all_tokens)?;

            let next_token = logits_processor
                .sample(&logits)
                .map_err(|e| LlmError::InferenceError(format!("Sampling: {e}")))?;

            if next_token == eos_token {
                break;
            }

            output_tokens.push(next_token);
            all_tokens.push(next_token);
        }

        // Decode output tokens
        let text = tokenizer
            .decode(&output_tokens, true)
            .map_err(|e| LlmError::TokenizerError(format!("Decode: {e}")))?;

        Ok(text.trim().to_string())
    }
}

/// Apply repeat penalty to logits for tokens that already appeared.
fn apply_repeat_penalty(logits: &Tensor, penalty: f32, tokens: &[u32]) -> Result<Tensor, LlmError> {
    let device = logits.device();
    let mut logits_vec: Vec<f32> = logits
        .to_vec1()
        .map_err(|e| LlmError::InferenceError(format!("To vec: {e}")))?;

    for &token in tokens {
        let idx = token as usize;
        if idx < logits_vec.len() {
            if logits_vec[idx] > 0.0 {
                logits_vec[idx] /= penalty;
            } else {
                logits_vec[idx] *= penalty;
            }
        }
    }

    Tensor::from_vec(logits_vec, logits.shape(), device)
        .map_err(|e| LlmError::InferenceError(format!("From vec: {e}")))
}
