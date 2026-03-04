use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("TimeDoctor authentication failed: {0}")]
    AuthFailed(String),

    #[error("Token expired — re-authentication required")]
    TokenExpired,

    #[error("Repository discovery failed: {0}")]
    RepoDiscovery(String),

    #[error("Sync script error (exit code {0})")]
    SyncScript(i32),

    #[error("Keyring error: {0}")]
    Keyring(String),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("fd binary not found — install with: brew install fd")]
    FdNotFound,
}
