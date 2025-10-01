use thiserror::Error;

/// Errors emitted while loading configuration from the environment.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("environment variable `{name}` is missing")]
    MissingEnvironment { name: &'static str },
    #[error("environment variable `{name}` is not valid unicode")]
    InvalidUnicode { name: &'static str },
    #[error("failed to parse server port from value `{value}`")]
    InvalidPort {
        value: String,
        #[source]
        source: std::num::ParseIntError,
    },
    #[error("failed to parse database max connections from value `{value}`")]
    InvalidMaxConnections {
        value: String,
        #[source]
        source: std::num::ParseIntError,
    },
    #[error("database max connections must be greater than zero (received {value})")]
    InvalidMaxConnectionsZero { value: u32 },
}
