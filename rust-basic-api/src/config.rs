//! Application configuration utilities.

use std::env;

use anyhow::{anyhow, Context, Result};

/// Global application configuration derived from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// `PostgreSQL` connection string consumed by `SQLx`.
    pub database_url: String,
    /// TCP port the HTTP server should listen on.
    pub server_port: u16,
}

impl Config {
    /// Load configuration values from environment variables, applying sensible defaults
    /// where appropriate.
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").context("DATABASE_URL environment variable must be set")?;

        let server_port = match env::var("SERVER_PORT") {
            Ok(value) => value
                .parse::<u16>()
                .context("SERVER_PORT must be a valid u16")?,
            Err(env::VarError::NotPresent) => 3000,
            Err(env::VarError::NotUnicode(_)) => {
                return Err(anyhow!("SERVER_PORT contains invalid UTF-8 characters"));
            }
        };

        Ok(Self {
            database_url,
            server_port,
        })
    }
}
