use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Installation error: {0}")]
    Installation(String),

    #[error("App not found: {0}")]
    AppNotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Controller error: {0}")]
    Controller(String),

    #[error("Optimization error: {0}")]
    Optimization(String),

    #[error("Backup error: {0}")]
    Backup(String),

    #[error("ZIP error: {0}")]
    Zip(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error("Desktop integration error: {0}")]
    DesktopIntegration(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

pub type Result<T> = std::result::Result<T, Error>;
