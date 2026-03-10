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
    /// Expected SHA256 hash of the GGUF file (hex string). None to skip verification.
    pub sha256_hash: Option<&'static str>,
}

pub const GEMMA2_2B: ModelConfig = ModelConfig {
    id: "gemma-2-2b-it-q4",
    display_name: "Gemma 2 2B (Q4_K_M)",
    repo_id: "bartowski/gemma-2-2b-it-GGUF",
    gguf_filename: "gemma-2-2b-it-Q4_K_M.gguf",
    tokenizer_filename: "tokenizer.json",
    file_size_mb: 1500,
    ram_requirement_mb: 2500,
    // TODO: fill in actual SHA256 hash after verifying the model file
    sha256_hash: None,
};

/// Manages model download, storage, and lifecycle.
///
/// Supports two-tier resolution: bundled models in `resource_dir` (read-only,
/// shipped with the app) and user-downloaded models in `models_dir` (writable).
pub struct ModelManager {
    models_dir: PathBuf,
    resource_dir: Option<PathBuf>,
}

impl ModelManager {
    pub fn new(app_data_dir: &Path, resource_dir: Option<PathBuf>) -> Self {
        let models_dir = app_data_dir.join("models");
        Self {
            models_dir,
            resource_dir,
        }
    }

    /// Path where a model's GGUF file is located.
    /// Checks bundled resource dir first, then user data dir.
    pub fn get_model_path(&self, config: &ModelConfig) -> PathBuf {
        if let Some(ref res) = self.resource_dir {
            let bundled = res.join(config.gguf_filename);
            if bundled.exists() {
                return bundled;
            }
        }
        self.models_dir
            .join(config.repo_id)
            .join(config.gguf_filename)
    }

    /// Path where a model's tokenizer is located.
    /// Checks bundled resource dir first, then user data dir.
    pub fn get_tokenizer_path(&self, config: &ModelConfig) -> PathBuf {
        if let Some(ref res) = self.resource_dir {
            let bundled = res.join(config.tokenizer_filename);
            if bundled.exists() {
                return bundled;
            }
        }
        self.models_dir
            .join(config.repo_id)
            .join(config.tokenizer_filename)
    }

    /// Check if a model's files are available (bundled or downloaded).
    ///
    /// Verifies both files exist and the GGUF file size is within 10% of the expected size.
    pub fn is_available(&self, config: &ModelConfig) -> bool {
        let model_path = self.get_model_path(config);
        let tokenizer_path = self.get_tokenizer_path(config);

        if !model_path.exists() || !tokenizer_path.exists() {
            return false;
        }

        // Verify model file isn't truncated (within 10% of expected size)
        match std::fs::metadata(&model_path) {
            Ok(metadata) => {
                let expected_bytes = config.file_size_mb as u64 * 1024 * 1024;
                if metadata.len() < expected_bytes * 9 / 10 {
                    return false;
                }
            }
            Err(_) => return false,
        }

        true
    }

    /// Verify the SHA256 hash of a file matches the expected value.
    ///
    /// Reads the file in 8KB chunks to avoid loading large files into memory.
    fn verify_hash(path: &Path, expected_hex: &str) -> Result<(), LlmError> {
        use sha2::{Digest, Sha256};
        use std::io::Read;

        let mut file = std::fs::File::open(path)
            .map_err(|e| LlmError::DownloadError(format!("Failed to open file for hash verification: {e}")))?;

        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];
        loop {
            let bytes_read = file
                .read(&mut buffer)
                .map_err(|e| LlmError::DownloadError(format!("Failed to read file for hash verification: {e}")))?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let actual_hex = format!("{:x}", hasher.finalize());
        if actual_hex != expected_hex {
            return Err(LlmError::DownloadError(format!(
                "SHA256 hash mismatch for {}: expected {expected_hex}, got {actual_hex}. \
                 The file may be corrupted — please delete it and re-download.",
                path.display()
            )));
        }

        Ok(())
    }

    /// Whether the model is available from the bundled resource directory.
    pub fn is_bundled(&self, config: &ModelConfig) -> bool {
        if let Some(ref res) = self.resource_dir {
            res.join(config.gguf_filename).exists()
                && res.join(config.tokenizer_filename).exists()
        } else {
            false
        }
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
        let target_model = self
            .models_dir
            .join(config.repo_id)
            .join(config.gguf_filename);
        if !target_model.exists() {
            if let Some(parent) = target_model.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| LlmError::DownloadError(format!("Create dir: {e}")))?;
            }
            std::fs::copy(&_model_path, &target_model)
                .map_err(|e| LlmError::DownloadError(format!("Copy model: {e}")))?;
        }

        // Verify SHA256 hash if configured
        if let Some(expected_hash) = config.sha256_hash {
            tracing::info!("Verifying SHA256 hash for {}", config.gguf_filename);
            Self::verify_hash(&target_model, expected_hash)?;
            tracing::info!("SHA256 hash verified for {}", config.gguf_filename);
        }

        // Download tokenizer
        tracing::info!("Downloading tokenizer: {}", config.tokenizer_filename);
        let _tokenizer_path = repo
            .download(config.tokenizer_filename)
            .await
            .map_err(|e| LlmError::DownloadError(format!("Tokenizer download: {e}")))?;

        let target_tokenizer = self
            .models_dir
            .join(config.repo_id)
            .join(config.tokenizer_filename);
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
    use std::io::Write;

    #[test]
    fn model_paths_without_resource_dir() {
        let mgr = ModelManager::new(Path::new("/tmp/test"), None);
        let path = mgr.get_model_path(&GEMMA2_2B);
        assert!(path.to_str().unwrap().contains("bartowski"));
        assert!(path.to_str().unwrap().ends_with(".gguf"));
    }

    #[test]
    fn tokenizer_path_separate_from_model() {
        let mgr = ModelManager::new(Path::new("/tmp/test"), None);
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
        assert!(GEMMA2_2B.sha256_hash.is_none()); // TODO: update when hash is known
    }

    #[test]
    fn is_available_returns_false_for_missing_files() {
        let tmp = tempfile::tempdir().unwrap();
        let mgr = ModelManager::new(tmp.path());
        assert!(!mgr.is_available(&GEMMA2_2B));
    }

    #[test]
    fn is_available_returns_false_for_truncated_model() {
        let tmp = tempfile::tempdir().unwrap();
        let mgr = ModelManager::new(tmp.path());

        let config = ModelConfig {
            id: "test",
            display_name: "Test",
            repo_id: "test/repo",
            gguf_filename: "model.gguf",
            tokenizer_filename: "tokenizer.json",
            file_size_mb: 1, // expect ~1MB
            ram_requirement_mb: 100,
            sha256_hash: None,
        };

        let model_path = mgr.get_model_path(&config);
        let tokenizer_path = mgr.get_tokenizer_path(&config);
        std::fs::create_dir_all(model_path.parent().unwrap()).unwrap();

        // Write a tiny file (way below 90% of 1MB)
        std::fs::write(&model_path, b"tiny").unwrap();
        std::fs::write(&tokenizer_path, b"{}").unwrap();

        assert!(!mgr.is_available(&config));
    }

    #[test]
    fn is_available_returns_true_for_adequate_size() {
        let tmp = tempfile::tempdir().unwrap();
        let mgr = ModelManager::new(tmp.path());

        let config = ModelConfig {
            id: "test",
            display_name: "Test",
            repo_id: "test/repo",
            gguf_filename: "model.gguf",
            tokenizer_filename: "tokenizer.json",
            file_size_mb: 1, // expect ~1MB
            ram_requirement_mb: 100,
            sha256_hash: None,
        };

        let model_path = mgr.get_model_path(&config);
        let tokenizer_path = mgr.get_tokenizer_path(&config);
        std::fs::create_dir_all(model_path.parent().unwrap()).unwrap();

        // Write a file that's at least 90% of 1MB
        let data = vec![0u8; 1024 * 1024]; // exactly 1MB
        std::fs::write(&model_path, &data).unwrap();
        std::fs::write(&tokenizer_path, b"{}").unwrap();

        assert!(mgr.is_available(&config));
    }

    #[test]
    fn verify_hash_succeeds_for_correct_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("test.bin");
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(b"hello world").unwrap();
        drop(f);

        // SHA256 of "hello world"
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert!(ModelManager::verify_hash(&file_path, expected).is_ok());
    }

    #[test]
    fn verify_hash_fails_for_wrong_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("test.bin");
        std::fs::write(&file_path, b"hello world").unwrap();

        let wrong = "0000000000000000000000000000000000000000000000000000000000000000";
        let result = ModelManager::verify_hash(&file_path, wrong);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("hash mismatch"));
    }

    #[test]
    fn bundled_resource_dir_preferred_when_exists() {
        // With a resource dir that doesn't have files, falls back to models_dir
        let mgr = ModelManager::new(Path::new("/tmp/test"), Some(PathBuf::from("/tmp/nonexistent")));
        let path = mgr.get_model_path(&GEMMA2_2B);
        // Should fall back to models_dir path since resource dir files don't exist
        assert!(path.to_str().unwrap().contains("bartowski"));
    }

    #[test]
    fn is_bundled_false_without_resource_dir() {
        let mgr = ModelManager::new(Path::new("/tmp/test"), None);
        assert!(!mgr.is_bundled(&GEMMA2_2B));
    }

    #[test]
    fn is_bundled_false_when_files_missing() {
        let mgr = ModelManager::new(Path::new("/tmp/test"), Some(PathBuf::from("/tmp/nonexistent")));
        assert!(!mgr.is_bundled(&GEMMA2_2B));
    }
}
