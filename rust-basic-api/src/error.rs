use thiserror::Error;

/// Application-level error type to unify error handling across layers.
#[allow(dead_code)] // Reserved for future error handling work.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Type alias for results returned by application components.
#[allow(dead_code)] // Reserved for future error handling work.
pub type Result<T> = std::result::Result<T, AppError>;
