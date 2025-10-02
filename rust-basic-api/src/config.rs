use dotenv::dotenv;
use std::{env, num::ParseIntError};
use thiserror::Error;

/// Application runtime configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable `{0}`")]
    MissingEnv(String),
    #[error("failed to parse `SERVER_PORT`: {0}")]
    InvalidPort(#[from] ParseIntError),
}

impl Config {
    /// Load configuration values from the process environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").map_err(|_| ConfigError::MissingEnv("DATABASE_URL".into()))?;

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()?;

        Ok(Self {
            database_url,
            server_port,
        })
    }
}
