//! App-side LLM policy: where the model lives on disk, ChessMentor's
//! generation settings, and UI metadata for the model picker.
//!
//! The runtime itself (download, load, generation) is the shared
//! `sensei-llm` crate; everything in this module is ChessMentor-specific
//! configuration around it.

use std::path::{Path, PathBuf};

use sensei_llm::{GEMMA4_E2B_Q4, GenerateOptions, ModelSpec, ModelStore, SamplerConfig};

/// The sampler values ChessMentor has always used. These happen to be
/// `sensei-llm`'s defaults (they were lifted from this app), but they are
/// pinned here so a future change to the shared defaults can never silently
/// alter coaching output.
pub const CHESS_SAMPLER: SamplerConfig = SamplerConfig {
    temperature: 0.3,
    top_k: Some(50),
    min_p: None,
    top_p: 0.9,
    repeat_penalty: 1.1,
    repeat_penalty_last_n: 64,
    seed: 42,
};

/// Context window size ChessMentor has always used. The shared default is
/// 1024; the coaching prompt (facts + engine lines) needs the larger window.
pub const CHESS_N_CTX: u32 = 2048;

/// Generation options for grammar-constrained coaching output.
///
/// - `n_ctx: 2048` keeps the pre-sensei-kit context size, preserving the
///   prompt-length guard and memory profile.
/// - `filter_channels: true` keeps the old always-on [`sensei_llm::ChannelFilter`]
///   behavior (set explicitly rather than relying on the shared default).
pub fn coaching_generate_options(max_tokens: u32, grammar: &str) -> GenerateOptions {
    GenerateOptions {
        max_tokens,
        n_ctx: CHESS_N_CTX,
        sampler: CHESS_SAMPLER,
        grammar: Some(grammar.to_string()),
        grammar_root: "root".to_string(),
        filter_channels: true,
    }
}

/// UI metadata for a downloadable model — app policy that used to live on
/// `mentor_llm::download::ModelConfig`, paired with the shared [`ModelSpec`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInfo {
    pub id: &'static str,
    pub display_name: &'static str,
    pub ram_requirement_mb: u32,
    pub spec: &'static ModelSpec,
}

impl ModelInfo {
    /// Published file size in whole MiB, rounded up (matches the 2963 the
    /// UI showed before the spec moved to exact bytes).
    pub fn file_size_mb(&self) -> u32 {
        u32::try_from(self.spec.size_bytes.div_ceil(1024 * 1024)).unwrap_or(u32::MAX)
    }
}

/// Gemma 4 E2B instruct, Q4_K_M — the model ChessMentor ships with.
pub const GEMMA4_E2B: ModelInfo = ModelInfo {
    id: "gemma-4-e2b-it-q4",
    display_name: "Gemma 4 E2B (Q4_K_M)",
    ram_requirement_mb: 4096,
    spec: &GEMMA4_E2B_Q4,
};

/// Get the model info by ID.
pub fn get_model_info(model_id: &str) -> Option<&'static ModelInfo> {
    match model_id {
        "gemma-4-e2b-it-q4" => Some(&GEMMA4_E2B),
        _ => None,
    }
}

/// Store for the coaching model, rooted at `{app_data}/models` with the
/// bundled-resource tier checked first.
///
/// No on-disk migration is needed: the old `mentor-llm` store already used
/// sensei-llm's exact layout — downloads at `{app_data}/models/{repo_id}/{filename}`,
/// hf-hub cache at `{app_data}/models/hf-cache`, and bundled resources
/// resolved as `{resource_dir}/{filename}` (see the layout parity test below).
pub fn model_store(app_data_dir: &Path, resource_dir: Option<PathBuf>) -> ModelStore {
    let store = ModelStore::new(app_data_dir.join("models"));
    match resource_dir {
        Some(dir) => store.with_resource_dir(dir),
        None => store,
    }
}

/// Name of the deprecated device-override environment variable.
const LEGACY_DEVICE_ENV: &str = "CHESS_MENTOR_DEVICE";
/// Name of the sensei-llm device-override environment variable.
const DEVICE_ENV: &str = "SENSEI_LLM_DEVICE";

/// Value the new device env var should be set to, given both current values.
///
/// The legacy variable only applies when the new one is unset, so an explicit
/// `SENSEI_LLM_DEVICE` always wins.
fn legacy_device_override(legacy: Option<String>, current: Option<String>) -> Option<String> {
    match current {
        Some(_) => None,
        None => legacy,
    }
}

/// Honor the deprecated `CHESS_MENTOR_DEVICE` variable by forwarding it to
/// `SENSEI_LLM_DEVICE` (which is all sensei-llm reads). Call once during app
/// setup, before any inference starts.
pub fn apply_legacy_device_env_fallback() {
    let legacy = std::env::var(LEGACY_DEVICE_ENV).ok();
    let current = std::env::var(DEVICE_ENV).ok();
    if let Some(value) = legacy_device_override(legacy, current) {
        tracing::warn!(
            "{LEGACY_DEVICE_ENV} is deprecated; use {DEVICE_ENV} instead \
             (forwarding {LEGACY_DEVICE_ENV}={value})"
        );
        // SAFETY: called once from Tauri's setup hook before any inference
        // thread reads the variable; no concurrent env access at this point.
        unsafe { std::env::set_var(DEVICE_ENV, &value) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chess_sampler_matches_historical_values() {
        assert_eq!(CHESS_SAMPLER.temperature, 0.3);
        assert_eq!(CHESS_SAMPLER.top_k, Some(50));
        assert_eq!(CHESS_SAMPLER.min_p, None);
        assert_eq!(CHESS_SAMPLER.top_p, 0.9);
        assert_eq!(CHESS_SAMPLER.repeat_penalty, 1.1);
        assert_eq!(CHESS_SAMPLER.repeat_penalty_last_n, 64);
        assert_eq!(CHESS_SAMPLER.seed, 42);
    }

    #[test]
    fn coaching_options_preserve_pre_kit_behavior() {
        let opts = coaching_generate_options(128, "root ::= [^<>]{1,600}\n");
        assert_eq!(opts.max_tokens, 128);
        assert_eq!(opts.n_ctx, 2048, "chess always ran with a 2048 context");
        assert_eq!(opts.sampler, CHESS_SAMPLER);
        assert_eq!(opts.grammar.as_deref(), Some("root ::= [^<>]{1,600}\n"));
        assert_eq!(opts.grammar_root, "root");
        assert!(opts.filter_channels, "chess always filtered channel blocks");
    }

    #[test]
    fn model_info_matches_old_ui_metadata() {
        assert_eq!(GEMMA4_E2B.id, "gemma-4-e2b-it-q4");
        assert_eq!(GEMMA4_E2B.display_name, "Gemma 4 E2B (Q4_K_M)");
        assert_eq!(GEMMA4_E2B.ram_requirement_mb, 4096);
        assert_eq!(GEMMA4_E2B.file_size_mb(), 2963);
        assert_eq!(GEMMA4_E2B.spec, &GEMMA4_E2B_Q4);
    }

    #[test]
    fn model_info_lookup() {
        assert!(get_model_info("gemma-4-e2b-it-q4").is_some());
        assert!(get_model_info("gemma-3-1b-it-q4").is_none());
        assert!(get_model_info("nonexistent").is_none());
    }

    /// The old `mentor_llm::download::ModelStore::new(app_data, resource)`
    /// resolved downloads to `{app_data}/models/{repo_id}/{filename}` and
    /// bundled models to `{resource_dir}/{filename}`. The sensei-llm store
    /// must resolve to the same paths so existing installs and the local dev
    /// `src-tauri/models/` resource dir keep working without migration.
    #[test]
    fn store_layout_matches_pre_kit_paths() {
        let app_data = Path::new("/tmp/chess-mentor-app-data");
        let store = model_store(app_data, None);
        assert_eq!(
            store.model_path(&GEMMA4_E2B_Q4),
            app_data
                .join("models")
                .join(GEMMA4_E2B_Q4.repo_id)
                .join(GEMMA4_E2B_Q4.filename)
        );
    }

    #[test]
    fn store_prefers_bundled_resource_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let res_dir = tmp.path().join("resources").join("models");
        std::fs::create_dir_all(&res_dir).expect("create resource dir");
        std::fs::write(res_dir.join(GEMMA4_E2B_Q4.filename), b"bundled").expect("write");

        let store = model_store(tmp.path(), Some(res_dir.clone()));
        assert_eq!(
            store.model_path(&GEMMA4_E2B_Q4),
            res_dir.join(GEMMA4_E2B_Q4.filename)
        );
        assert!(store.is_bundled(&GEMMA4_E2B_Q4));
    }

    #[test]
    fn legacy_device_var_forwards_only_when_new_var_unset() {
        assert_eq!(
            legacy_device_override(Some("cpu".into()), None),
            Some("cpu".into())
        );
        assert_eq!(
            legacy_device_override(Some("cpu".into()), Some("metal".into())),
            None,
            "an explicit SENSEI_LLM_DEVICE must win over the legacy var"
        );
        assert_eq!(legacy_device_override(None, None), None);
        assert_eq!(legacy_device_override(None, Some("cpu".into())), None);
    }
}
