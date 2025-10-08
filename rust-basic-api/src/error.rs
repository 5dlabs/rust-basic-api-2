//! Error types and application-wide result aliases.

use std::num::ParseIntError;

use thiserror::Error;

/// Convenient result alias for fallible application operations.
pub type AppResult<T> = anyhow::Result<T>;

/// Errors that can occur while loading environment-driven configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The requested environment variable was missing.
    #[error("environment variable `{name}` is not set")]
    MissingEnvVar { name: &'static str },

    /// The environment variable could not be parsed into the expected type.
    #[error("environment variable `{name}` has an invalid value: {source}")]
    InvalidEnvVar {
        name: &'static str,
        #[source]
        source: ParseIntError,
    },

    /// The environment variable contained invalid UTF-8 data.
    #[error("environment variable `{name}` contains invalid unicode data")]
    InvalidUnicode { name: &'static str },
}
