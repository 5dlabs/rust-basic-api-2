//! Application configuration handling.

use std::env;

use dotenv::dotenv;

use crate::error::ConfigError;

/// Core runtime configuration for the service.
#[derive(Debug, Clone)]
pub struct Config {
    /// `PostgreSQL` connection string.
    pub database_url: String,
    /// TCP port the HTTP server listens on.
    pub server_port: u16,
}

impl Config {
    /// Construct a [`Config`] instance by reading environment variables.
    ///
    /// The following variables are read:
    /// - `DATABASE_URL` (required)
    /// - `SERVER_PORT` (optional, defaults to `3000`)
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when required environment variables are missing
    /// or contain invalid data.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url = match env::var("DATABASE_URL") {
            Ok(value) => value,
            Err(env::VarError::NotPresent) => {
                return Err(ConfigError::Missing {
                    name: "DATABASE_URL",
                });
            }
            Err(source) => {
                return Err(ConfigError::Environment {
                    name: "DATABASE_URL",
                    source,
                });
            }
        };

        let server_port = match env::var("SERVER_PORT") {
            Ok(value) => value
                .parse::<u16>()
                .map_err(|source| ConfigError::InvalidNumber {
                    name: "SERVER_PORT",
                    value,
                    source,
                })?,
            Err(env::VarError::NotPresent) => 3000,
            Err(source) => {
                return Err(ConfigError::Environment {
                    name: "SERVER_PORT",
                    source,
                });
            }
        };

        Ok(Self {
            database_url,
            server_port,
        })
    }
}
