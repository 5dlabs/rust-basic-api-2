//! Application configuration management.

use std::{env, net::IpAddr};

use dotenv::dotenv;

use crate::error::ConfigError;

/// Runtime configuration derived from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Database connection string compatible with `PostgreSQL`.
    pub database_url: String,
    /// Host interface on which the server will listen.
    pub server_host: IpAddr,
    /// TCP port used by the HTTP server.
    pub server_port: u16,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Required environment variables:
    ///
    /// - `DATABASE_URL`
    ///
    /// Optional variables:
    ///
    /// - `SERVER_HOST` (defaults to `0.0.0.0`)
    /// - `SERVER_PORT` (defaults to `3000`)
    /// - `RUST_LOG` (handled by the tracing subscriber)
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").map_err(|_| ConfigError::Missing("DATABASE_URL"))?;

        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string())
            .parse()
            .map_err(|err| ConfigError::invalid("SERVER_HOST", err))?;

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .map_err(|err| ConfigError::invalid("SERVER_PORT", err))?;

        Ok(Self {
            database_url,
            server_host,
            server_port,
        })
    }
}
