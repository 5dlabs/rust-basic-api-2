use std::num::ParseIntError;

use thiserror::Error;

/// Errors that can occur while building application configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Required environment variable is missing.
    #[error("environment variable `{name}` is not set")]
    MissingEnvVar { name: &'static str },

    /// Environment variable contains invalid (non-UTF-8) data.
    #[error("environment variable `{name}` contains invalid UTF-8 data")]
    InvalidUnicode { name: &'static str },

    /// Provided server port could not be parsed into a valid `u16` value.
    #[error("invalid server port `{port}`: {source}")]
    InvalidServerPort {
        port: String,
        #[source]
        source: ParseIntError,
    },
}
