use std::env;

use dotenv::dotenv;

use crate::error::ConfigError;

const DEFAULT_SERVER_PORT: u16 = 3000;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    /// Loads configuration values from environment variables.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if a required environment variable is missing or invalid.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").map_err(|source| ConfigError::MissingEnvVar {
                name: "DATABASE_URL".to_string(),
                source,
            })?;

        let server_port = match env::var("SERVER_PORT") {
            Ok(value) => value
                .trim()
                .parse::<u16>()
                .map_err(|source| ConfigError::InvalidPort { value, source })?,
            Err(_) => DEFAULT_SERVER_PORT,
        };

        Ok(Self {
            database_url,
            server_port,
        })
    }
}
