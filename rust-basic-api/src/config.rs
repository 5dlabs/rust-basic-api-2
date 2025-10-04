use std::{env, num::ParseIntError};

use dotenv::dotenv;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    /// Load configuration values from the environment.
    ///
    /// The `DATABASE_URL` variable is required. `SERVER_PORT` is optional and
    /// defaults to `3000` when not provided.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url = get_required_env("DATABASE_URL")?;
        let server_port = parse_server_port(env::var("SERVER_PORT").ok())?;

        Ok(Self {
            database_url,
            server_port,
        })
    }
}

fn get_required_env(key: &str) -> Result<String, ConfigError> {
    match env::var(key) {
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => {
            Err(ConfigError::MissingEnvironmentVariable(key.to_string()))
        }
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::InvalidUnicode(key.to_string())),
    }
}

fn parse_server_port(value: Option<String>) -> Result<u16, ConfigError> {
    match value {
        Some(port) => port
            .parse::<u16>()
            .map_err(|source| ConfigError::InvalidServerPort { port, source }),
        None => Ok(3000),
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable `{0}`")]
    MissingEnvironmentVariable(String),
    #[error("environment variable `{0}` contains invalid unicode")]
    InvalidUnicode(String),
    #[error("failed to parse server port `{port}`: {source}")]
    InvalidServerPort {
        port: String,
        #[source]
        source: ParseIntError,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn defaults_to_standard_port() {
        let _guard = ENV_MUTEX.lock().expect("mutex poisoned");
        reset_env();

        set_env_var("DATABASE_URL", "postgresql://example.com/db");
        let config = Config::from_env().expect("configuration should load");

        assert_eq!(config.server_port, 3000);
        assert_eq!(config.database_url, "postgresql://example.com/db");

        reset_env();
    }

    #[test]
    fn parses_custom_port_value() {
        let _guard = ENV_MUTEX.lock().expect("mutex poisoned");
        reset_env();

        set_env_var("DATABASE_URL", "postgresql://example.com/db");
        set_env_var("SERVER_PORT", "8080");
        let config = Config::from_env().expect("configuration should load");

        assert_eq!(config.server_port, 8080);

        reset_env();
    }

    #[test]
    fn errors_when_database_url_missing() {
        let _guard = ENV_MUTEX.lock().expect("mutex poisoned");
        reset_env();

        let error = Config::from_env().expect_err("config should fail without database url");

        assert!(matches!(
            error,
            ConfigError::MissingEnvironmentVariable(var) if var == "DATABASE_URL"
        ));

        reset_env();
    }

    fn reset_env() {
        remove_env_var("DATABASE_URL");
        remove_env_var("SERVER_PORT");
    }

    fn set_env_var(key: &str, value: &str) {
        // Mutating environment variables is considered unsafe in Rust 2024 because it
        // can lead to data races across threads. Our tests serialize access via a
        // mutex, making this operation safe in practice.
        unsafe {
            env::set_var(key, value);
        }
    }

    fn remove_env_var(key: &str) {
        unsafe {
            env::remove_var(key);
        }
    }
}
