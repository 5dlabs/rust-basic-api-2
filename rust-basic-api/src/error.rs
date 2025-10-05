//! Error handling types for the application.

use anyhow::Error as AnyError;
use sqlx::{migrate::MigrateError, Error as SqlxError};
use thiserror::Error;

/// Result alias that uses [`AppError`] as the error type.
pub type AppResult<T> = Result<T, AppError>;

/// Top-level application error enumeration.
#[derive(Debug, Error)]
pub enum AppError {
    /// Wraps a configuration-specific error.
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// Represents database-related failures.
    #[error(transparent)]
    Database(#[from] SqlxError),

    /// Represents other runtime failures propagated through [`anyhow::Error`].
    #[error(transparent)]
    Runtime(#[from] AnyError),
}

impl From<MigrateError> for AppError {
    fn from(error: MigrateError) -> Self {
        Self::Runtime(error.into())
    }
}

/// Errors emitted while loading configuration from the environment.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Required environment variable was not set.
    #[error("missing required environment variable `{0}`")]
    Missing(&'static str),

    /// Environment variable contained an invalid value.
    #[error("invalid value for `{field}`: {source}")]
    InvalidEnvVar {
        /// Name of the offending environment variable.
        field: &'static str,
        /// Source error describing why parsing failed.
        #[source]
        source: AnyError,
    },
}

impl ConfigError {
    /// Helper for creating an [`InvalidEnvVar`](Self::InvalidEnvVar) error from any source error.
    #[must_use]
    pub fn invalid(field: &'static str, source: impl Into<AnyError>) -> Self {
        Self::InvalidEnvVar {
            field,
            source: source.into(),
        }
    }
}
