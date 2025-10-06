use std::env;

use dotenv::dotenv;

use crate::error::ConfigError;

/// Application configuration populated from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// The `DATABASE_URL` variable is required. `SERVER_PORT` is optional and
    /// defaults to `3000` when not provided.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url = read_env_var("DATABASE_URL")?;
        let server_port = match env::var("SERVER_PORT") {
            Ok(raw) => parse_port(raw)?,
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

fn read_env_var(name: &'static str) -> Result<String, ConfigError> {
    match env::var(name) {
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => Err(ConfigError::MissingEnvVar { name }),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::InvalidUnicode { name }),
    }
}

fn parse_port(raw: String) -> Result<u16, ConfigError> {
    raw.parse::<u16>()
        .map_err(|source| ConfigError::InvalidServerPort { port: raw, source })
}
