use anyhow::{Context, Result};
use dotenv::dotenv;
use std::env;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    /// Load configuration values from the current environment.
    ///
    /// Falls back to sensible defaults where appropriate and returns an error
    /// if required values are missing or malformed.
    pub fn from_env() -> Result<Self> {
        dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").context("DATABASE_URL environment variable is required")?;

        let server_port = env::var("SERVER_PORT")
            .ok()
            .map(|value| {
                value
                    .parse::<u16>()
                    .context("SERVER_PORT must be a valid unsigned 16-bit integer")
            })
            .transpose()? // propagate parsing error if present
            .unwrap_or(3000);

        Ok(Self {
            database_url,
            server_port,
        })
    }
}
