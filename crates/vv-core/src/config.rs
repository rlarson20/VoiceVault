use crate::error::{VvError, VvResult};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

/// Main configuration structure
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    /// Vault configuration
    pub vault: VaultConfig,
    /// Speech-to-text configuration
    pub stt: SttConfig,
    /// Trigger configuration
    pub trigger: TriggerConfig,
    /// Audio configuration
    pub audio: AudioConfig,
}

/// Vault-specific configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct VaultConfig {
    /// Path to the vault directory
    pub path: PathBuf,
    /// Folder name within vault for recordings
    pub folder: String,
    /// Filename format (strftime-compatible, e.g., "%Y-%m-%dT%H-%M-%S")
    pub filename_format: String,
}

/// Speech-to-text configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SttConfig {
    /// Whisper model name (tiny, base, small, medium, large)
    pub model: String,
}

/// Trigger configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct TriggerConfig {
    /// Whether push-to-talk is enabled
    pub push_to_talk: bool,
    /// Push-to-talk key combination
    pub push_key: String,
    /// Whether continuous listening is enabled
    pub continuous: bool,
    /// Voice activation keyword (e.g., "hey vault")
    pub keyword: String,
}

/// Audio configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AudioConfig {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of audio channels
    pub channels: u16,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            path: default_vault_path(),
            folder: default_folder(),
            filename_format: default_filename_format(),
        }
    }
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            model: default_model(),
        }
    }
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            push_to_talk: true,
            push_key: String::from("Ctrl+Shift+V"),
            continuous: false,
            keyword: String::from("hey vault"),
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
        }
    }
}

/// Default folder name for recordings
fn default_folder() -> String {
    String::from("voice-notes")
}

/// Default filename format (ISO 8601-like timestamp)
fn default_filename_format() -> String {
    String::from("%Y-%m-%dT%H-%M-%S")
}

/// Default Whisper model
fn default_model() -> String {
    String::from("base")
}

/// Default vault path (~/Documents/Vault or ~/Documents)
fn default_vault_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join("Documents").join("Vault")
}

/// Configuration loader with search path logic
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from multiple sources in priority order:
    /// 1. CLI-provided path
    /// 2. VOICEVAULT_CONFIG environment variable
    /// 3. XDG config directory
    /// 4. Return error if not found
    pub fn load(cli_path: Option<PathBuf>) -> VvResult<Config> {
        let config_path = Self::find_config_path(cli_path)?;

        info!("Loading configuration from: {}", config_path.display());
        let content = std::fs::read_to_string(&config_path).map_err(|e| {
            VvError::ConfigParse(format!("Failed to read {}: {}", config_path.display(), e))
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            VvError::ConfigParse(format!("Failed to parse {}: {}", config_path.display(), e))
        })?;

        // Validate the configuration
        Self::validate(&config)?;

        Ok(config)
    }

    /// Find configuration file path following the priority order
    fn find_config_path(cli_path: Option<PathBuf>) -> VvResult<PathBuf> {
        // 1. Check CLI-provided path
        if let Some(path) = cli_path {
            if path.exists() {
                return Ok(path);
            }
            return Err(VvError::ConfigNotFound(path));
        }

        // 2. Check environment variable
        if let Ok(env_path) = std::env::var("VOICEVAULT_CONFIG") {
            let path = PathBuf::from(env_path);
            if path.exists() {
                return Ok(path);
            }
            warn!(
                "VOICEVAULT_CONFIG points to non-existent file: {}",
                path.display()
            );
        }

        // 3. Check XDG config directory
        if let Some(config_dir) = dirs::config_dir() {
            let xdg_path = config_dir.join("voicevault").join("config.toml");
            if xdg_path.exists() {
                return Ok(xdg_path);
            }
        }

        // 4. Return error if not found
        Err(VvError::Config("Configuration file not found. Please place config.toml in XDG config directory (~/.config/voicevault/) or specify path with --config".to_string()))
    }

    /// Validate configuration values
    pub fn validate(config: &Config) -> VvResult<()> {
        // Validate vault path
        if !config.vault.path.exists() {
            return Err(VvError::VaultNotFound(config.vault.path.clone()));
        }

        if !config.vault.path.is_dir() {
            return Err(VvError::Config(format!(
                "Vault path is not a directory: {}",
                config.vault.path.display()
            )));
        }

        // Validate model name
        let valid_models = ["tiny", "base", "small", "medium", "large"];
        if !valid_models.contains(&config.stt.model.as_str()) {
            return Err(VvError::UnknownModel(config.stt.model.clone()));
        }

        // Validate trigger configuration
        if config.trigger.push_to_talk && config.trigger.continuous {
            warn!("Both push-to-talk and continuous listening are enabled");
        }

        if !config.trigger.push_to_talk && !config.trigger.continuous {
            return Err(VvError::Config(
                "At least one trigger mode must be enabled".to_string(),
            ));
        }

        // Validate audio configuration
        if config.audio.sample_rate == 0 {
            return Err(VvError::Config(
                "Sample rate must be greater than 0".to_string(),
            ));
        }

        if config.audio.channels == 0 {
            return Err(VvError::Config(
                "Channel count must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}
