use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum VvError {
    #[error("Config error: {0}")]
    Config(String),

    #[error("Config file not found at: {0}")]
    ConfigNotFound(PathBuf),

    #[error("Failed to parse config file: {0}")]
    ConfigParse(String),

    #[error("Vault path does not exist or is not accessible: {0}")]
    VaultNotFound(PathBuf),

    #[error("Vault is not writable: {0}")]
    VaultNotWritable(PathBuf),

    #[error("Vault write failed: {0}")]
    VaultWrite(String),

    #[error("Transcription failed: {0}")]
    Transcription(String),

    #[error("Audio capture error: {0}")]
    AudioCapture(String),

    #[error("Trigger error: {0}")]
    Trigger(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unknown model: {0}")]
    UnknownModel(String),
}

pub type VvResult<T> = std::result::Result<T, VvError>;
