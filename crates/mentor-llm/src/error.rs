/// Errors from the LLM subsystem.
#[derive(Debug, Clone, thiserror::Error)]
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
}
