use std::path::Path;

use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_gemma3::ModelWeights;
use tokenizers::Tokenizer;
use tokio_util::sync::CancellationToken;

use super::LlmError;

/// Select the compute device for inference.
///
/// Default: CUDA if the `llm-cuda` feature is enabled and a GPU is present,
/// otherwise CPU. Metal is compiled in on macOS but **opt-in** via
/// `CHESS_MENTOR_DEVICE=metal`: candle 0.9's quantized-gemma3 Metal path is
/// orders of magnitude slower than CPU on low-memory Apple Silicon (measured
/// 783s vs 3.2s for the same generation on an 8 GB M3 — the 262k-vocab output
/// projection appears to thrash), while CPU comfortably meets the app's
/// coaching timeouts. `CHESS_MENTOR_DEVICE=cpu` forces CPU everywhere.
pub fn select_device() -> (Device, &'static str) {
    match std::env::var("CHESS_MENTOR_DEVICE").as_deref() {
        Ok("cpu") => {
            tracing::info!("CHESS_MENTOR_DEVICE=cpu — forcing CPU inference");
            return (Device::Cpu, "cpu");
        }
        Ok("metal") => {
            #[cfg(any(feature = "llm-metal", target_os = "macos"))]
            match Device::new_metal(0) {
                Ok(device) => {
                    tracing::info!("CHESS_MENTOR_DEVICE=metal — using Metal GPU for inference");
                    return (device, "metal");
                }
                Err(e) => tracing::warn!("Metal initialization failed: {e}, falling back"),
            }
            #[cfg(not(any(feature = "llm-metal", target_os = "macos")))]
            tracing::warn!("CHESS_MENTOR_DEVICE=metal requested but Metal support not compiled");
        }
        _ => {}
    }

    #[cfg(feature = "llm-cuda")]
    {
        match Device::cuda_if_available(0) {
            Ok(device) if device.is_cuda() => {
                tracing::info!("Using CUDA GPU for inference");
                return (device, "cuda");
            }
            Ok(_) => tracing::info!("CUDA requested but no GPU available, falling back to CPU"),
            Err(e) => tracing::warn!("CUDA initialization failed: {e}, falling back to CPU"),
        }
    }

    tracing::info!("Using CPU for inference");
    (Device::Cpu, "cpu")
}

/// Return a display name for a device.
pub fn device_name(device: &Device) -> &'static str {
    match device {
        Device::Cpu => "cpu",
        Device::Cuda(_) => "cuda",
        Device::Metal(_) => "metal",
    }
}

/// Candle-based inference backend for quantized Gemma 3 GGUF models.
pub struct CandleBackend {
    model: Option<ModelWeights>,
    tokenizer: Option<Tokenizer>,
    device: Device,
}

impl CandleBackend {
    pub fn new(device: Device) -> Self {
        Self {
            model: None,
            tokenizer: None,
            device,
        }
    }

    /// Load a quantized GGUF model and tokenizer from disk.
    pub fn load(&mut self, model_path: &Path, tokenizer_path: &Path) -> Result<(), LlmError> {
        let mut model_file = std::fs::File::open(model_path)
            .map_err(|e| LlmError::InferenceError(format!("Failed to open model: {e}")))?;

        let content = candle_core::quantized::gguf_file::Content::read(&mut model_file)
            .map_err(|e| LlmError::InferenceError(format!("Failed to read GGUF: {e}")))?;

        let model = ModelWeights::from_gguf(content, &mut model_file, &self.device)
            .map_err(|e| LlmError::InferenceError(format!("Failed to load model weights: {e}")))?;

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| LlmError::TokenizerError(format!("Failed to load tokenizer: {e}")))?;

        self.model = Some(model);
        self.tokenizer = Some(tokenizer);

        tracing::info!(
            "Candle backend loaded model from {} on {}",
            model_path.display(),
            device_name(&self.device)
        );
        Ok(())
    }

    /// Generate text from a prompt using token-by-token decoding.
    ///
    /// Supports cancellation via the provided token.
    /// Calls `on_token` with each new chunk of decoded text as it becomes available.
    /// Uses temperature 0.3, top_p 0.9, repeat_penalty 1.1, max 128 tokens.
    pub fn generate<F>(
        &mut self,
        prompt: &str,
        cancel: &CancellationToken,
        mut on_token: F,
    ) -> Result<String, LlmError>
    where
        F: FnMut(&str),
    {
        let model = self.model.as_mut().ok_or(LlmError::ModelNotLoaded)?;
        let tokenizer = self.tokenizer.as_ref().ok_or(LlmError::ModelNotLoaded)?;

        let encoding = tokenizer
            .encode(prompt, true)
            .map_err(|e| LlmError::TokenizerError(e.to_string()))?;
        let prompt_tokens = encoding.get_ids().to_vec();

        if prompt_tokens.is_empty() {
            return Err(LlmError::InferenceError("Empty prompt".to_string()));
        }

        let mut logits_processor = LogitsProcessor::from_sampling(
            42,
            candle_transformers::generation::Sampling::TopKThenTopP {
                k: 50,
                p: 0.9,
                temperature: 0.3,
            },
        );

        // Gemma instruction models end responses with <end_of_turn>, not <eos>,
        // so treat every known terminator as a stop token.
        let stop_tokens: Vec<u32> = ["<eos>", "<end_of_turn>", "</s>"]
            .iter()
            .filter_map(|t| tokenizer.token_to_id(t))
            .collect();
        let is_stop =
            |token: u32| stop_tokens.contains(&token) || (stop_tokens.is_empty() && token == 1);

        let max_tokens: usize = 128;
        let mut output_tokens: Vec<u32> = Vec::with_capacity(max_tokens);
        let mut all_tokens = prompt_tokens.clone();
        // Track how many characters we've already emitted for incremental decoding
        let mut prev_decoded_len: usize = 0;

        // Process prompt tokens as a batch first
        let input = Tensor::new(prompt_tokens.as_slice(), &self.device)
            .map_err(|e| LlmError::InferenceError(format!("Tensor creation: {e}")))?
            .unsqueeze(0)
            .map_err(|e| LlmError::InferenceError(format!("Unsqueeze: {e}")))?;

        // forward() narrows to the last position internally and returns [batch, vocab]
        let logits = model
            .forward(&input, 0)
            .map_err(|e| LlmError::InferenceError(format!("Forward pass: {e}")))?;

        let logits = logits
            .squeeze(0)
            .map_err(|e| LlmError::InferenceError(format!("Squeeze: {e}")))?;

        let logits = apply_repeat_penalty(&logits, 1.1, &all_tokens)?;

        let mut next_token = logits_processor
            .sample(&logits)
            .map_err(|e| LlmError::InferenceError(format!("Sampling: {e}")))?;

        if is_stop(next_token) {
            return Ok(String::new());
        }

        output_tokens.push(next_token);
        all_tokens.push(next_token);

        // Emit first token incrementally
        emit_new_text(
            tokenizer,
            &output_tokens,
            &mut prev_decoded_len,
            &mut on_token,
        );

        // Generate remaining tokens one at a time
        for i in 0..max_tokens - 1 {
            if cancel.is_cancelled() {
                return Err(LlmError::Cancelled);
            }

            let input = Tensor::new(&[next_token], &self.device)
                .map_err(|e| LlmError::InferenceError(format!("Tensor: {e}")))?
                .unsqueeze(0)
                .map_err(|e| LlmError::InferenceError(format!("Unsqueeze: {e}")))?;

            let pos = prompt_tokens.len() + i;
            let logits = model
                .forward(&input, pos)
                .map_err(|e| LlmError::InferenceError(format!("Forward: {e}")))?;

            let logits = logits
                .squeeze(0)
                .map_err(|e| LlmError::InferenceError(format!("Squeeze: {e}")))?;

            let logits = apply_repeat_penalty(&logits, 1.1, &all_tokens)?;

            next_token = logits_processor
                .sample(&logits)
                .map_err(|e| LlmError::InferenceError(format!("Sampling: {e}")))?;

            if is_stop(next_token) {
                break;
            }

            output_tokens.push(next_token);
            all_tokens.push(next_token);

            emit_new_text(
                tokenizer,
                &output_tokens,
                &mut prev_decoded_len,
                &mut on_token,
            );
        }

        // Decode full output
        let text = tokenizer
            .decode(&output_tokens, true)
            .map_err(|e| LlmError::TokenizerError(format!("Decode: {e}")))?;

        Ok(text.trim().to_string())
    }
}

/// Decode the full token buffer and emit only the newly produced characters.
///
/// This avoids emitting partial UTF-8 sequences by always decoding the complete
/// buffer and only sending the delta since the last successful decode.
fn emit_new_text<F: FnMut(&str)>(
    tokenizer: &Tokenizer,
    tokens: &[u32],
    prev_len: &mut usize,
    on_token: &mut F,
) {
    if let Ok(decoded) = tokenizer.decode(tokens, true)
        && decoded.len() > *prev_len
    {
        on_token(&decoded[*prev_len..]);
        *prev_len = decoded.len();
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

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end inference smoke test against the real GGUF.
    ///
    /// Requires model files in src-tauri/models/ (run scripts/fetch-model.sh first).
    /// Run with: cargo test --release -- --ignored llm_generates_coherent_text --nocapture
    #[test]
    #[ignore]
    fn llm_generates_coherent_text() {
        let models_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("models");
        let model_path = models_dir.join(super::super::model_manager::GEMMA3_1B.gguf_filename);
        let tokenizer_path =
            models_dir.join(super::super::model_manager::GEMMA3_1B.tokenizer_filename);
        assert!(
            model_path.exists() && tokenizer_path.exists(),
            "model files missing — run scripts/fetch-model.sh first"
        );

        let (device, dev_name) = select_device();
        eprintln!("inference device: {dev_name}");
        let mut backend = CandleBackend::new(device);
        let t0 = std::time::Instant::now();
        backend
            .load(&model_path, &tokenizer_path)
            .expect("model should load");
        eprintln!("load took {:.1}s", t0.elapsed().as_secs_f32());

        let prompt = "<start_of_turn>user\nReply with one short sentence: why is controlling \
                      the center important in chess?<end_of_turn>\n<start_of_turn>model\n";
        let cancel = CancellationToken::new();
        let mut streamed = String::new();
        let mut chunks = 0u32;
        let t1 = std::time::Instant::now();
        let text = backend
            .generate(prompt, &cancel, |t| {
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
            !text.contains("<end_of_turn>") && !text.contains("<start_of_turn>"),
            "output leaked turn markers: {text:?}"
        );
        assert_eq!(
            streamed.trim(),
            text,
            "streamed text should match final text"
        );
    }
}
