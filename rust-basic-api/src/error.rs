use std::num::ParseIntError;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("environment variable `{key}` is required but missing")]
    MissingEnv { key: String },
    #[error("environment variable `{key}` contained invalid unicode characters")]
    InvalidUnicode { key: String },
    #[error("environment variable `{key}` cannot be empty")]
    EmptyEnv { key: String },
    #[error("environment variable `{key}` could not be parsed: {source}")]
    InvalidValue {
        key: String,
        #[source]
        source: ParseIntError,
    },
    #[error("environment variable `{key}` must be at least {min}")]
    InvalidRange { key: String, min: u32 },
}
