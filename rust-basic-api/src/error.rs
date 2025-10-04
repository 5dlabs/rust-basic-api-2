use std::{env::VarError, num::ParseIntError};

use thiserror::Error;

/// Convenience result type for application errors.
pub type AppResult<T> = Result<T, anyhow::Error>;

/// Errors that can occur while loading configuration values.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Required environment variable is missing.
    #[error("missing environment variable `{name}`")]
    MissingEnvVar {
        name: String,
        #[source]
        source: VarError,
    },

    /// Provided server port value cannot be parsed into an integer.
    #[error("invalid server port `{value}`")]
    InvalidPort {
        value: String,
        #[source]
        source: ParseIntError,
    },
}
