//! Application configuration loaded from the environment.

use std::env;

use dotenv::dotenv;

use crate::error::ConfigError;

/// Application configuration values sourced from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// `PostgreSQL` connection string used by the repository layer.
    #[allow(dead_code)]
    // The database URL will be consumed by repository initialization in upcoming tasks.
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    /// Load configuration from environment variables, falling back to sensible defaults where allowed.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url = match env::var("DATABASE_URL") {
            Ok(value) => value,
            Err(env::VarError::NotPresent) => {
                return Err(ConfigError::MissingEnvVar {
                    name: "DATABASE_URL",
                });
            }
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::InvalidUnicode {
                    name: "DATABASE_URL",
                });
            }
        };

        let server_port = match env::var("SERVER_PORT") {
            Ok(value) => value
                .parse::<u16>()
                .map_err(|source| ConfigError::InvalidEnvVar {
                    name: "SERVER_PORT",
                    source,
                })?,
            Err(env::VarError::NotPresent) => 3000,
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::InvalidUnicode {
                    name: "SERVER_PORT",
                });
            }
        };

        Ok(Self {
            database_url,
            server_port,
        })
    }
}
