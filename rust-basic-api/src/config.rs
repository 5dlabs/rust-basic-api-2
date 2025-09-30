use crate::error::ConfigError;
use dotenv::dotenv;
use std::env;

/// Application configuration derived from the process environment.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    /// Construct configuration by reading required environment variables.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url = match env::var("DATABASE_URL") {
            Ok(value) => value,
            Err(env::VarError::NotPresent) => {
                return Err(ConfigError::MissingEnvironment {
                    name: "DATABASE_URL",
                })
            }
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::InvalidUnicode {
                    name: "DATABASE_URL",
                })
            }
        };

        let server_port = match env::var("SERVER_PORT") {
            Ok(value) => value
                .parse::<u16>()
                .map_err(|source| ConfigError::InvalidPort { value, source })?,
            Err(env::VarError::NotPresent) => 3000,
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::InvalidUnicode {
                    name: "SERVER_PORT",
                })
            }
        };

        Ok(Self {
            database_url,
            server_port,
        })
    }
}
