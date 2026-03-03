#![allow(dead_code)]

use std::path::{Path, PathBuf};

use super::LlmError;

/// Configuration for a downloadable model.
pub struct ModelConfig {
    pub id: &'static str,
    pub display_name: &'static str,
    pub repo_id: &'static str,
    pub gguf_filename: &'static str,
    pub tokenizer_filename: &'static str,
    pub file_size_mb: u32,
    pub ram_requirement_mb: u32,
}

pub const GEMMA2_2B: ModelConfig = ModelConfig {
    id: "gemma-2-2b-it-q4",
    display_name: "Gemma 2 2B (Q4_K_M)",
    repo_id: "bartowski/gemma-2-2b-it-GGUF",
    gguf_filename: "gemma-2-2b-it-Q4_K_M.gguf",
    tokenizer_filename: "tokenizer.json",
    file_size_mb: 1500,
    ram_requirement_mb: 2500,
};

/// Manages model download, storage, and lifecycle.
pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new(app_data_dir: &Path) -> Self {
        let models_dir = app_data_dir.join("models");
        Self { models_dir }
    }

    /// Path where a model's GGUF file would be stored.
    pub fn get_model_path(&self, config: &ModelConfig) -> PathBuf {
        self.models_dir
            .join(config.repo_id)
            .join(config.gguf_filename)
    }

    /// Path where a model's tokenizer would be stored.
    pub fn get_tokenizer_path(&self, config: &ModelConfig) -> PathBuf {
        self.models_dir
            .join(config.repo_id)
            .join(config.tokenizer_filename)
    }

    /// Check if a model's files are already downloaded.
    pub fn is_available(&self, config: &ModelConfig) -> bool {
        self.get_model_path(config).exists() && self.get_tokenizer_path(config).exists()
    }

    /// Download a model from HuggingFace Hub.
    ///
    /// Emits `llm-download-progress` events via the app handle.
    pub async fn download(
        &self,
        config: &ModelConfig,
        app_handle: &tauri::AppHandle,
    ) -> Result<(), LlmError> {
        use hf_hub::api::tokio::ApiBuilder;
        use tauri::Emitter;

        let model_dir = self.models_dir.join(config.repo_id);
        std::fs::create_dir_all(&model_dir)
            .map_err(|e| LlmError::DownloadError(format!("Failed to create dir: {e}")))?;

        let api = ApiBuilder::new()
            .with_cache_dir(self.models_dir.clone())
            .build()
            .map_err(|e| LlmError::DownloadError(format!("HF Hub API init: {e}")))?;

        let repo = api.model(config.repo_id.to_string());

        // Download GGUF model file
        tracing::info!("Downloading model: {}", config.gguf_filename);
        let handle = app_handle.clone();
        let _model_path = repo
            .download(config.gguf_filename)
            .await
            .map_err(|e| LlmError::DownloadError(format!("Model download: {e}")))?;

        // Copy/symlink to our expected location if not already there
        let target_model = self.get_model_path(config);
        if !target_model.exists() {
            if let Some(parent) = target_model.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| LlmError::DownloadError(format!("Create dir: {e}")))?;
            }
            std::fs::copy(&_model_path, &target_model)
                .map_err(|e| LlmError::DownloadError(format!("Copy model: {e}")))?;
        }

        // Download tokenizer
        tracing::info!("Downloading tokenizer: {}", config.tokenizer_filename);
        let _tokenizer_path = repo
            .download(config.tokenizer_filename)
            .await
            .map_err(|e| LlmError::DownloadError(format!("Tokenizer download: {e}")))?;

        let target_tokenizer = self.get_tokenizer_path(config);
        if !target_tokenizer.exists() {
            std::fs::copy(&_tokenizer_path, &target_tokenizer)
                .map_err(|e| LlmError::DownloadError(format!("Copy tokenizer: {e}")))?;
        }

        let _ = handle.emit(
            "llm-download-progress",
            serde_json::json!({
                "downloadedBytes": config.file_size_mb as u64 * 1024 * 1024,
                "totalBytes": config.file_size_mb as u64 * 1024 * 1024,
            }),
        );

        tracing::info!("Model download complete: {}", config.id);
        Ok(())
    }

    /// Delete a downloaded model's files.
    pub fn delete_model(&self, config: &ModelConfig) -> Result<(), LlmError> {
        let model_dir = self.models_dir.join(config.repo_id);
        if model_dir.exists() {
            std::fs::remove_dir_all(&model_dir)
                .map_err(|e| LlmError::DownloadError(format!("Delete failed: {e}")))?;
        }
        tracing::info!("Deleted model: {}", config.id);
        Ok(())
    }

    /// Get the model config by ID.
    pub fn get_config(model_id: &str) -> Option<&'static ModelConfig> {
        match model_id {
            "gemma-2-2b-it-q4" => Some(&GEMMA2_2B),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_paths_are_constructed_correctly() {
        let mgr = ModelManager::new(Path::new("/tmp/test"));
        let path = mgr.get_model_path(&GEMMA2_2B);
        assert!(path.to_str().unwrap().contains("bartowski"));
        assert!(path.to_str().unwrap().ends_with(".gguf"));
    }

    #[test]
    fn tokenizer_path_separate_from_model() {
        let mgr = ModelManager::new(Path::new("/tmp/test"));
        let model_path = mgr.get_model_path(&GEMMA2_2B);
        let tok_path = mgr.get_tokenizer_path(&GEMMA2_2B);
        assert_ne!(model_path, tok_path);
        assert!(tok_path.to_str().unwrap().ends_with("tokenizer.json"));
    }

    #[test]
    fn model_config_lookup() {
        assert!(ModelManager::get_config("gemma-2-2b-it-q4").is_some());
        assert!(ModelManager::get_config("nonexistent").is_none());
    }

    #[test]
    fn gemma2_constants_are_reasonable() {
        assert_eq!(GEMMA2_2B.file_size_mb, 1500);
        assert_eq!(GEMMA2_2B.ram_requirement_mb, 2500);
        assert!(!GEMMA2_2B.repo_id.is_empty());
    }
}
