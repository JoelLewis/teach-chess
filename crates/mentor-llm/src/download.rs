use std::path::{Path, PathBuf};

use tracing::info;

use crate::error::LlmError;

/// Configuration for a downloadable model.
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub id: &'static str,
    pub display_name: &'static str,
    pub repo_id: &'static str,
    pub gguf_filename: &'static str,
    pub file_size_mb: u32,
    pub ram_requirement_mb: u32,
    /// Expected SHA256 hash of the GGUF file (hex string). None to skip verification.
    pub sha256_hash: Option<&'static str>,
}

/// Gemma 4 E2B instruct, Q4_K_M quantization (3,106,736,256 bytes).
///
/// llama.cpp embeds the tokenizer in the GGUF, so no separate tokenizer
/// download is needed (unlike the old candle + tokenizers stack).
pub const GEMMA4_E2B: ModelConfig = ModelConfig {
    id: "gemma-4-e2b-it-q4",
    display_name: "Gemma 4 E2B (Q4_K_M)",
    repo_id: "unsloth/gemma-4-E2B-it-GGUF",
    gguf_filename: "gemma-4-E2B-it-Q4_K_M.gguf",
    file_size_mb: 2963,
    ram_requirement_mb: 4096,
    sha256_hash: Some("9378bc471710229ef165709b62e34bfb62231420ddaf6d729e727305b5b8672d"),
};

/// Get the model config by ID.
pub fn get_config(model_id: &str) -> Option<&'static ModelConfig> {
    match model_id {
        "gemma-4-e2b-it-q4" => Some(&GEMMA4_E2B),
        _ => None,
    }
}

/// Adapts hf-hub's [`hf_hub::api::Progress`] to a `(downloaded, total)` callback.
struct CallbackProgress<F: FnMut(u64, u64)> {
    downloaded: u64,
    total: u64,
    on_progress: F,
}

impl<F: FnMut(u64, u64)> hf_hub::api::Progress for CallbackProgress<F> {
    fn init(&mut self, size: usize, _filename: &str) {
        self.total = size as u64;
        (self.on_progress)(0, self.total);
    }

    fn update(&mut self, size: usize) {
        self.downloaded += size as u64;
        (self.on_progress)(self.downloaded, self.total);
    }

    fn finish(&mut self) {
        (self.on_progress)(self.total, self.total);
    }
}

/// Resolves and downloads model files.
///
/// Supports two-tier resolution: bundled models in `resource_dir` (read-only,
/// shipped with the app) and user-downloaded models in `models_dir` (writable).
#[derive(Debug, Clone)]
pub struct ModelStore {
    models_dir: PathBuf,
    resource_dir: Option<PathBuf>,
}

impl ModelStore {
    pub fn new(app_data_dir: &Path, resource_dir: Option<PathBuf>) -> Self {
        let models_dir = app_data_dir.join("models");
        Self {
            models_dir,
            resource_dir,
        }
    }

    /// Path where a model's GGUF file is located.
    /// Checks bundled resource dir first, then user data dir.
    pub fn model_path(&self, config: &ModelConfig) -> PathBuf {
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

    /// Check if a model's GGUF file is available (bundled or downloaded).
    ///
    /// Verifies the file exists and its size is within 10% of the expected size.
    pub fn is_available(&self, config: &ModelConfig) -> bool {
        let model_path = self.model_path(config);

        match std::fs::metadata(&model_path) {
            Ok(metadata) => {
                let expected_bytes = config.file_size_mb as u64 * 1024 * 1024;
                metadata.len() >= expected_bytes * 9 / 10
            }
            Err(_) => false,
        }
    }

    /// Whether the model is available from the bundled resource directory.
    pub fn is_bundled(&self, config: &ModelConfig) -> bool {
        self.resource_dir
            .as_ref()
            .is_some_and(|res| res.join(config.gguf_filename).exists())
    }

    /// Download a model GGUF from HuggingFace Hub.
    ///
    /// Progress is reported via callback as `(bytes_downloaded, total_bytes)`.
    /// This is a blocking operation — call from `spawn_blocking`.
    pub fn download(
        &self,
        config: &ModelConfig,
        on_progress: impl FnMut(u64, u64),
    ) -> Result<PathBuf, LlmError> {
        let target = self
            .models_dir
            .join(config.repo_id)
            .join(config.gguf_filename);

        if target.exists() {
            info!("Model already cached at {}", target.display());
            return Ok(target);
        }

        info!(
            "Downloading model from {}/{}",
            config.repo_id, config.gguf_filename
        );

        let target_dir = target
            .parent()
            .expect("model target path always has a parent");
        std::fs::create_dir_all(target_dir)
            .map_err(|e| LlmError::DownloadError(format!("create model dir: {e}")))?;

        let api = hf_hub::api::sync::ApiBuilder::new()
            .with_cache_dir(self.models_dir.join("hf-cache"))
            .build()
            .map_err(|e| LlmError::DownloadError(format!("HF API init: {e}")))?;

        let repo = api.model(config.repo_id.to_string());
        let progress = CallbackProgress {
            downloaded: 0,
            total: config.file_size_mb as u64 * 1024 * 1024,
            on_progress,
        };

        let downloaded_path = repo
            .download_with_progress(config.gguf_filename, progress)
            .map_err(|e| LlmError::DownloadError(format!("download: {e}")))?;

        info!("Model downloaded to {}", downloaded_path.display());

        if let Some(expected_hash) = config.sha256_hash {
            info!("Verifying SHA256 hash for {}", config.gguf_filename);
            if let Err(e) = verify_hash(&downloaded_path, expected_hash) {
                let _ = std::fs::remove_file(&downloaded_path);
                return Err(e);
            }
            info!("SHA256 hash verified for {}", config.gguf_filename);
        }

        // hf-hub caches files in its own snapshot structure; link to the
        // stable path that `model_path` resolves.
        if downloaded_path != target && !target.exists() {
            link_or_copy(&downloaded_path, &target)?;
        }

        info!("Model download complete: {}", config.id);
        Ok(target)
    }
}

#[cfg(unix)]
fn link_or_copy(src: &Path, dest: &Path) -> Result<(), LlmError> {
    std::os::unix::fs::symlink(src, dest)
        .map_err(|e| LlmError::DownloadError(format!("symlink: {e}")))
}

#[cfg(not(unix))]
fn link_or_copy(src: &Path, dest: &Path) -> Result<(), LlmError> {
    std::fs::copy(src, dest)
        .map(|_| ())
        .map_err(|e| LlmError::DownloadError(format!("copy: {e}")))
}

/// Verify the SHA256 hash of a file matches the expected value.
///
/// Reads the file in 8KB chunks to avoid loading large files into memory.
fn verify_hash(path: &Path, expected_hex: &str) -> Result<(), LlmError> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file = std::fs::File::open(path).map_err(|e| {
        LlmError::DownloadError(format!("Failed to open file for hash verification: {e}"))
    })?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = file.read(&mut buffer).map_err(|e| {
            LlmError::DownloadError(format!("Failed to read file for hash verification: {e}"))
        })?;
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

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CONFIG: ModelConfig = ModelConfig {
        id: "test",
        display_name: "Test",
        repo_id: "test/repo",
        gguf_filename: "model.gguf",
        file_size_mb: 1,
        ram_requirement_mb: 100,
        sha256_hash: None,
    };

    #[test]
    fn model_path_without_resource_dir() {
        let store = ModelStore::new(Path::new("/tmp/test"), None);
        let path = store.model_path(&GEMMA4_E2B);
        assert!(path.to_str().unwrap().contains("unsloth"));
        assert!(path.to_str().unwrap().ends_with(".gguf"));
    }

    #[test]
    fn model_config_lookup() {
        assert!(get_config("gemma-4-e2b-it-q4").is_some());
        assert!(get_config("gemma-3-1b-it-q4").is_none());
        assert!(get_config("nonexistent").is_none());
    }

    #[test]
    fn gemma4_constants_are_reasonable() {
        assert_eq!(GEMMA4_E2B.file_size_mb, 2963);
        assert_eq!(GEMMA4_E2B.ram_requirement_mb, 4096);
        assert!(!GEMMA4_E2B.repo_id.is_empty());
        assert_eq!(GEMMA4_E2B.sha256_hash.unwrap().len(), 64);
    }

    #[test]
    fn is_available_returns_false_for_missing_files() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ModelStore::new(tmp.path(), None);
        assert!(!store.is_available(&GEMMA4_E2B));
    }

    #[test]
    fn is_available_returns_false_for_truncated_model() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ModelStore::new(tmp.path(), None);

        let model_path = store.model_path(&TEST_CONFIG);
        std::fs::create_dir_all(model_path.parent().unwrap()).unwrap();

        // Write a tiny file (way below 90% of 1MB)
        std::fs::write(&model_path, b"tiny").unwrap();

        assert!(!store.is_available(&TEST_CONFIG));
    }

    #[test]
    fn is_available_returns_true_for_adequate_size() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ModelStore::new(tmp.path(), None);

        let model_path = store.model_path(&TEST_CONFIG);
        std::fs::create_dir_all(model_path.parent().unwrap()).unwrap();

        // Write a file that's at least 90% of 1MB
        let data = vec![0u8; 1024 * 1024];
        std::fs::write(&model_path, &data).unwrap();

        assert!(store.is_available(&TEST_CONFIG));
    }

    #[test]
    fn download_returns_cached_path_without_network() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ModelStore::new(tmp.path(), None);

        let model_path = store.model_path(&TEST_CONFIG);
        std::fs::create_dir_all(model_path.parent().unwrap()).unwrap();
        std::fs::write(&model_path, b"fake gguf data").unwrap();

        let result = store.download(&TEST_CONFIG, |_, _| {});
        assert_eq!(result.unwrap(), model_path);
    }

    #[test]
    fn verify_hash_succeeds_for_correct_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("test.bin");
        std::fs::write(&file_path, b"hello world").unwrap();

        // SHA256 of "hello world"
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert!(verify_hash(&file_path, expected).is_ok());
    }

    #[test]
    fn verify_hash_fails_for_wrong_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("test.bin");
        std::fs::write(&file_path, b"hello world").unwrap();

        let wrong = "0000000000000000000000000000000000000000000000000000000000000000";
        let result = verify_hash(&file_path, wrong);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("hash mismatch"));
    }

    #[test]
    fn bundled_resource_dir_preferred_when_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let res_dir = tmp.path().join("resources");
        std::fs::create_dir_all(&res_dir).unwrap();
        std::fs::write(res_dir.join(TEST_CONFIG.gguf_filename), b"bundled").unwrap();

        let store = ModelStore::new(tmp.path(), Some(res_dir.clone()));
        assert_eq!(
            store.model_path(&TEST_CONFIG),
            res_dir.join(TEST_CONFIG.gguf_filename)
        );
        assert!(store.is_bundled(&TEST_CONFIG));
    }

    #[test]
    fn is_bundled_false_without_resource_dir() {
        let store = ModelStore::new(Path::new("/tmp/test"), None);
        assert!(!store.is_bundled(&GEMMA4_E2B));
    }

    #[test]
    fn is_bundled_false_when_files_missing() {
        let store = ModelStore::new(
            Path::new("/tmp/test"),
            Some(PathBuf::from("/tmp/nonexistent")),
        );
        assert!(!store.is_bundled(&GEMMA4_E2B));
    }
}
