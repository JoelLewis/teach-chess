#![cfg(feature = "llm")]
#![allow(dead_code)]

use std::path::Path;

use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use super::candle_backend::CandleBackend;
use super::LlmError;

/// A job submitted to the inference worker.
struct InferenceJob {
    prompt: String,
    response_tx: oneshot::Sender<Result<String, LlmError>>,
    cancel: CancellationToken,
}

/// Bounded channel for sequencing LLM inference requests.
///
/// Owns a background worker task that processes jobs one at a time.
/// Each new submission automatically cancels any in-flight request.
pub struct InferenceChannel {
    request_tx: mpsc::Sender<InferenceJob>,
    current_cancel: Option<CancellationToken>,
}

impl InferenceChannel {
    /// Spawn the inference worker. The backend must be loaded before submitting jobs.
    pub fn spawn(model_path: &Path, tokenizer_path: &Path) -> Result<Self, LlmError> {
        let (request_tx, mut request_rx) = mpsc::channel::<InferenceJob>(4);

        let model_path = model_path.to_path_buf();
        let tokenizer_path = tokenizer_path.to_path_buf();

        tokio::spawn(async move {
            // Load the model in the background worker
            let mut backend = CandleBackend::new();
            if let Err(e) = backend.load(&model_path, &tokenizer_path) {
                tracing::error!("Failed to load model in worker: {e}");
                // Drain any pending jobs with error responses
                while let Some(job) = request_rx.recv().await {
                    let _ = job.response_tx.send(Err(LlmError::ModelNotLoaded));
                }
                return;
            }

            tracing::info!("Inference worker ready");

            while let Some(job) = request_rx.recv().await {
                let prompt = job.prompt;
                let cancel = job.cancel;

                // Run inference on a blocking thread since it's CPU-bound
                let result = {
                    // We need &mut backend, but spawn_blocking requires 'static.
                    // Use a channel to send the backend into the blocking task and get it back.
                    let (backend_tx, backend_rx) = oneshot::channel();
                    let (result_tx, result_rx) = oneshot::channel();

                    // Temporarily take ownership of backend
                    let mut moved_backend = std::mem::replace(&mut backend, CandleBackend::new());

                    let cancel_clone = cancel.clone();
                    tokio::task::spawn_blocking(move || {
                        let result = moved_backend.generate(&prompt, &cancel_clone);
                        let _ = result_tx.send(result);
                        let _ = backend_tx.send(moved_backend);
                    });

                    // Get backend back
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

        Ok(Self {
            request_tx,
            current_cancel: None,
        })
    }

    /// Submit a prompt for inference.
    ///
    /// Cancels any previously in-flight request.
    /// Returns a receiver for the result.
    pub async fn submit(
        &mut self,
        prompt: String,
    ) -> Result<oneshot::Receiver<Result<String, LlmError>>, LlmError> {
        // Cancel previous request
        if let Some(old_cancel) = self.current_cancel.take() {
            old_cancel.cancel();
        }

        let cancel = CancellationToken::new();
        self.current_cancel = Some(cancel.clone());

        let (response_tx, response_rx) = oneshot::channel();

        let job = InferenceJob {
            prompt,
            response_tx,
            cancel,
        };

        self.request_tx
            .send(job)
            .await
            .map_err(|_| LlmError::InferenceError("Worker channel closed".to_string()))?;

        Ok(response_rx)
    }

    /// Whether the channel is still connected to the worker.
    pub fn is_alive(&self) -> bool {
        !self.request_tx.is_closed()
    }
}
