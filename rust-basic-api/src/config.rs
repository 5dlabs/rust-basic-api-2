use dotenv::dotenv;
use std::{env, num::ParseIntError};
use thiserror::Error;

/// Application runtime configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable `{0}`")]
    MissingEnv(String),
    #[error("failed to parse `SERVER_PORT`: {0}")]
    InvalidPort(#[from] ParseIntError),
}

impl Config {
    /// Load configuration values from the process environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").map_err(|_| ConfigError::MissingEnv("DATABASE_URL".into()))?;

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()?;

        Ok(Self {
            database_url,
            server_port,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, sync::Mutex};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_env() {
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    fn from_env_loads_defaults() {
        let _guard = ENV_LOCK.lock().expect("mutex poisoned");

        clear_env();
        env::set_var("DATABASE_URL", "postgresql://localhost:5432/example_db");

        let config = Config::from_env().expect("configuration should load");

        assert_eq!(
            config.database_url,
            "postgresql://localhost:5432/example_db"
        );
        assert_eq!(config.server_port, 3000);

        clear_env();
    }

    #[test]
    fn from_env_respects_port_override() {
        let _guard = ENV_LOCK.lock().expect("mutex poisoned");

        clear_env();
        env::set_var("DATABASE_URL", "postgresql://localhost:5432/example_db");
        env::set_var("SERVER_PORT", "8080");

        let config = Config::from_env().expect("configuration should load");

        assert_eq!(config.server_port, 8080);

        clear_env();
    }

    #[test]
    fn from_env_requires_database_url() {
        let _guard = ENV_LOCK.lock().expect("mutex poisoned");

        clear_env();

        let error = Config::from_env().expect_err("configuration should fail without database url");

        assert!(matches!(error, ConfigError::MissingEnv(name) if name == "DATABASE_URL"));

        clear_env();
    }
}
