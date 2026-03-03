use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Active visual theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Study,
    Grid,
    System,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Study => write!(f, "study"),
            Theme::Grid => write!(f, "grid"),
            Theme::System => write!(f, "system"),
        }
    }
}

/// Audio configuration persisted in TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    #[serde(default = "default_volume")]
    pub master_volume: f64,
    #[serde(default = "default_ambient_volume")]
    pub ambient_volume: f64,
    #[serde(default = "default_volume")]
    pub sfx_volume: f64,
    #[serde(default = "default_volume")]
    pub notification_volume: f64,
    #[serde(default)]
    pub muted: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: default_volume(),
            ambient_volume: default_ambient_volume(),
            sfx_volume: default_volume(),
            notification_volume: default_volume(),
            muted: false,
        }
    }
}

fn default_volume() -> f64 {
    1.0
}

fn default_ambient_volume() -> f64 {
    0.5
}

/// Top-level app configuration stored as TOML.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub audio: AudioConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_theme")]
    pub active: Theme,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            active: default_theme(),
        }
    }
}

fn default_theme() -> Theme {
    Theme::Study
}

/// Managed wrapper that holds config + path for persistence.
pub struct AppConfigState {
    config: AppConfig,
    path: PathBuf,
}

impl AppConfigState {
    /// Load config from disk, or create default if missing.
    pub fn load(app_data_dir: &Path) -> Self {
        let path = app_data_dir.join("config.toml");
        let config = if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
                    tracing::warn!("Invalid config.toml, using defaults: {e}");
                    AppConfig::default()
                }),
                Err(e) => {
                    tracing::warn!("Failed to read config.toml, using defaults: {e}");
                    AppConfig::default()
                }
            }
        } else {
            AppConfig::default()
        };

        Self { config, path }
    }

    /// Save current config to disk.
    fn save(&self) -> Result<(), String> {
        let contents =
            toml::to_string_pretty(&self.config).map_err(|e| format!("Serialize error: {e}"))?;

        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {e}"))?;
        }

        std::fs::write(&self.path, contents)
            .map_err(|e| format!("Failed to write config.toml: {e}"))?;

        Ok(())
    }

    pub fn get_theme(&self) -> Theme {
        self.config.theme.active
    }

    pub fn set_theme(&mut self, theme: Theme) -> Result<(), String> {
        self.config.theme.active = theme;
        self.save()
    }

    pub fn get_audio(&self) -> &AudioConfig {
        &self.config.audio
    }

    pub fn set_audio(&mut self, audio: AudioConfig) -> Result<(), String> {
        self.config.audio = audio;
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.theme.active, Theme::Study);
        assert!((config.audio.master_volume - 1.0).abs() < f64::EPSILON);
        assert!((config.audio.ambient_volume - 0.5).abs() < f64::EPSILON);
        assert!(!config.audio.muted);
    }

    #[test]
    fn test_theme_display() {
        assert_eq!(Theme::Study.to_string(), "study");
        assert_eq!(Theme::Grid.to_string(), "grid");
        assert_eq!(Theme::System.to_string(), "system");
    }

    #[test]
    fn test_theme_serde_roundtrip() {
        let toml_str = r#"
[theme]
active = "grid"

[audio]
master_volume = 0.8
ambient_volume = 0.3
sfx_volume = 1.0
notification_volume = 0.9
muted = true
"#;
        let config: AppConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.theme.active, Theme::Grid);
        assert!((config.audio.master_volume - 0.8).abs() < f64::EPSILON);
        assert!(config.audio.muted);

        // Roundtrip
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.theme.active, Theme::Grid);
    }

    #[test]
    fn test_load_missing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let state = AppConfigState::load(tmp.path());
        assert_eq!(state.get_theme(), Theme::Study);
    }

    #[test]
    fn test_save_and_reload() {
        let tmp = tempfile::tempdir().unwrap();
        let mut state = AppConfigState::load(tmp.path());
        state.set_theme(Theme::Grid).unwrap();

        let reloaded = AppConfigState::load(tmp.path());
        assert_eq!(reloaded.get_theme(), Theme::Grid);
    }

    #[test]
    fn test_partial_toml_uses_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "[theme]\nactive = \"grid\"").unwrap();

        let state = AppConfigState::load(tmp.path());
        assert_eq!(state.get_theme(), Theme::Grid);
        assert!((state.get_audio().master_volume - 1.0).abs() < f64::EPSILON);
    }
}
