use crate::error::ConfigError;
use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url = require_env("DATABASE_URL")?;
        let server_port = optional_port("SERVER_PORT")?;

        Ok(Self {
            database_url,
            server_port,
        })
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub fn server_port(&self) -> u16 {
        self.server_port
    }
}

fn require_env(key: &str) -> Result<String, ConfigError> {
    match env::var(key) {
        Ok(value) => {
            if value.trim().is_empty() {
                Err(ConfigError::EmptyEnv {
                    key: key.to_string(),
                })
            } else {
                Ok(value)
            }
        }
        Err(env::VarError::NotPresent) => Err(ConfigError::MissingEnv {
            key: key.to_string(),
        }),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::InvalidUnicode {
            key: key.to_string(),
        }),
    }
}

fn optional_port(key: &str) -> Result<u16, ConfigError> {
    match env::var(key) {
        Ok(value) => {
            if value.trim().is_empty() {
                return Err(ConfigError::EmptyEnv {
                    key: key.to_string(),
                });
            }

            value
                .parse::<u16>()
                .map_err(|source| ConfigError::InvalidValue {
                    key: key.to_string(),
                    source,
                })
        }
        Err(env::VarError::NotPresent) => Ok(3000),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::InvalidUnicode {
            key: key.to_string(),
        }),
    }
}
