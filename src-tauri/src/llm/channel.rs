#![allow(dead_code)]

use std::path::Path;

use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use super::candle_backend::{select_device, CandleBackend};
use super::LlmError;

/// A job submitted to the inference worker.
struct InferenceJob {
    prompt: String,
    response_tx: oneshot::Sender<Result<String, LlmError>>,
    cancel: CancellationToken,
    token_tx: Option<mpsc::UnboundedSender<String>>,
}

/// Result of submitting a job: the final-result receiver and a token stream receiver.
pub struct SubmitResult {
    pub response_rx: oneshot::Receiver<Result<String, LlmError>>,
    pub token_rx: mpsc::UnboundedReceiver<String>,
}

/// Bounded channel for sequencing LLM inference requests.
///
/// Owns a background worker task that processes jobs one at a time.
/// Each new submission automatically cancels any in-flight request.
/// Duplicate requests (same prompt) are deduplicated if one is in-flight.
pub struct InferenceChannel {
    request_tx: mpsc::Sender<InferenceJob>,
    current_cancel: Option<CancellationToken>,
    /// Tracks the prompt of the currently in-flight request for deduplication.
    in_flight_prompt: Option<String>,
    in_flight_rx: Option<tokio::sync::watch::Receiver<Option<Result<String, LlmError>>>>,
    /// Which compute device this channel uses.
    pub device_name: String,
}

impl InferenceChannel {
    /// Spawn the inference worker with automatic device selection.
    ///
    /// Tries GPU (CUDA/Metal) first, falls back to CPU.
    pub fn spawn(model_path: &Path, tokenizer_path: &Path) -> Result<(Self, String), LlmError> {
        let (request_tx, mut request_rx) = mpsc::channel::<InferenceJob>(4);

        let model_path = model_path.to_path_buf();
        let tokenizer_path = tokenizer_path.to_path_buf();

        let (device, dev_name) = select_device();
        let device_name = dev_name.to_string();

        // We need the device for the placeholder backend too
        let placeholder_dev_name = dev_name;

        tokio::spawn(async move {
            let mut backend = CandleBackend::new(device);
            if let Err(e) = backend.load(&model_path, &tokenizer_path) {
                tracing::error!("Failed to load model in worker: {e}");
                while let Some(job) = request_rx.recv().await {
                    let _ = job.response_tx.send(Err(LlmError::ModelNotLoaded));
                }
                return;
            }

            tracing::info!("Inference worker ready on {placeholder_dev_name}");

            while let Some(job) = request_rx.recv().await {
                let prompt = job.prompt;
                let cancel = job.cancel;
                let token_tx = job.token_tx;

                let result = {
                    let (backend_tx, backend_rx) = oneshot::channel();
                    let (result_tx, result_rx) = oneshot::channel();

                    // Temporarily move backend into blocking task.
                    // The placeholder uses CPU since it's never used for inference.
                    let mut moved_backend =
                        std::mem::replace(&mut backend, CandleBackend::new(candle_core::Device::Cpu));

                    let cancel_clone = cancel.clone();
                    tokio::task::spawn_blocking(move || {
                        let result = moved_backend.generate(&prompt, &cancel_clone, |text| {
                            if let Some(ref tx) = token_tx {
                                let _ = tx.send(text.to_string());
                            }
                        });
                        let _ = result_tx.send(result);
                        let _ = backend_tx.send(moved_backend);
                    });

                    if let Ok(returned_backend) = backend_rx.await {
                        backend = returned_backend;
                    }

                    result_rx.await.unwrap_or(Err(LlmError::InferenceError(
                        "Worker task panicked".to_string(),
                    )))
                };

                let _ = job.response_tx.send(result);
            }

            tracing::info!("Inference worker shutting down");
        });

        let channel = Self {
            request_tx,
            current_cancel: None,
            in_flight_prompt: None,
            in_flight_rx: None,
            device_name: device_name.clone(),
        };

        Ok((channel, device_name))
    }

    /// Submit a prompt for inference.
    ///
    /// Returns a `SubmitResult` with both the final-text receiver and a token stream.
    /// Deduplicated requests get a dummy (immediately-closed) token receiver.
    pub async fn submit(&mut self, prompt: String) -> Result<SubmitResult, LlmError> {
        // Deduplication: if the same prompt is already in-flight, return a proxy receiver
        if let Some(ref existing_prompt) = self.in_flight_prompt {
            if *existing_prompt == prompt {
                if let Some(ref rx) = self.in_flight_rx {
                    let mut watch_rx = rx.clone();
                    let (proxy_tx, proxy_rx) = oneshot::channel();
                    tokio::spawn(async move {
                        while watch_rx.changed().await.is_ok() {
                            if let Some(result) = watch_rx.borrow().as_ref() {
                                let _ = proxy_tx.send(result.clone());
                                return;
                            }
                        }
                        let _ = proxy_tx.send(Err(LlmError::InferenceError(
                            "Dedup channel closed".to_string(),
                        )));
                    });
                    // Dedup'd requests skip streaming — return a closed token receiver
                    let (_dummy_tx, dummy_rx) = mpsc::unbounded_channel();
                    return Ok(SubmitResult {
                        response_rx: proxy_rx,
                        token_rx: dummy_rx,
                    });
                }
            }
        }

        // Cancel previous request
        if let Some(old_cancel) = self.current_cancel.take() {
            old_cancel.cancel();
        }

        let cancel = CancellationToken::new();
        self.current_cancel = Some(cancel.clone());

        let (response_tx, response_rx) = oneshot::channel();

        // Set up dedup tracking via a watch channel
        let (watch_tx, watch_rx) = tokio::sync::watch::channel(None);
        self.in_flight_prompt = Some(prompt.clone());
        self.in_flight_rx = Some(watch_rx);

        let (actual_tx, actual_rx) = oneshot::channel::<Result<String, LlmError>>();
        tokio::spawn(async move {
            if let Ok(result) = actual_rx.await {
                let _ = watch_tx.send(Some(result.clone()));
                let _ = response_tx.send(result);
            }
        });

        // Token streaming channel
        let (token_tx, token_rx) = mpsc::unbounded_channel();

        let job = InferenceJob {
            prompt,
            response_tx: actual_tx,
            cancel,
            token_tx: Some(token_tx),
        };

        self.request_tx
            .send(job)
            .await
            .map_err(|_| LlmError::InferenceError("Worker channel closed".to_string()))?;

        Ok(SubmitResult {
            response_rx,
            token_rx,
        })
    }

    /// Whether the channel is still connected to the worker.
    pub fn is_alive(&self) -> bool {
        !self.request_tx.is_closed()
    }
}
