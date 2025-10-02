use crate::repository::{
    DEFAULT_ACQUIRE_TIMEOUT_SECS, DEFAULT_IDLE_TIMEOUT_SECS, DEFAULT_MAX_CONNECTIONS,
    DEFAULT_MIN_CONNECTIONS,
};
use dotenv::dotenv;
use std::{env, num::ParseIntError};
use thiserror::Error;

/// Application runtime configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_min_connections: u32,
    pub database_acquire_timeout_secs: u64,
    pub database_idle_timeout_secs: u64,
    pub server_port: u16,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable `{0}`")]
    MissingEnv(String),
    #[error("failed to parse `{name}`: {source}")]
    InvalidNumber {
        name: String,
        #[source]
        source: ParseIntError,
    },
    #[error("environment variable `{0}` contained invalid unicode")]
    InvalidUnicode(String),
}

impl Config {
    /// Load configuration values from the process environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url = match env::var("DATABASE_URL") {
            Ok(value) => value,
            Err(env::VarError::NotPresent) => {
                return Err(ConfigError::MissingEnv("DATABASE_URL".into()))
            }
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::InvalidUnicode("DATABASE_URL".into()))
            }
        };

        let database_max_connections =
            parse_env_or_default("DATABASE_MAX_CONNECTIONS", DEFAULT_MAX_CONNECTIONS)?;
        let database_min_connections =
            parse_env_or_default("DATABASE_MIN_CONNECTIONS", DEFAULT_MIN_CONNECTIONS)?;
        let database_acquire_timeout_secs = parse_env_or_default(
            "DATABASE_ACQUIRE_TIMEOUT_SECS",
            DEFAULT_ACQUIRE_TIMEOUT_SECS,
        )?;
        let database_idle_timeout_secs =
            parse_env_or_default("DATABASE_IDLE_TIMEOUT_SECS", DEFAULT_IDLE_TIMEOUT_SECS)?;

        let server_port = parse_env_or_default("SERVER_PORT", 3000u16)?;

        Ok(Self {
            database_url,
            database_max_connections,
            database_min_connections,
            database_acquire_timeout_secs,
            database_idle_timeout_secs,
            server_port,
        })
    }
}

fn parse_env_or_default<T>(name: &str, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr<Err = ParseIntError>,
{
    match env::var(name) {
        Ok(value) => value.parse().map_err(|source| ConfigError::InvalidNumber {
            name: name.into(),
            source,
        }),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::InvalidUnicode(name.into())),
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
        env::remove_var("DATABASE_MAX_CONNECTIONS");
        env::remove_var("DATABASE_MIN_CONNECTIONS");
        env::remove_var("DATABASE_ACQUIRE_TIMEOUT_SECS");
        env::remove_var("DATABASE_IDLE_TIMEOUT_SECS");
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
        assert_eq!(config.database_max_connections, DEFAULT_MAX_CONNECTIONS);
        assert_eq!(config.database_min_connections, DEFAULT_MIN_CONNECTIONS);
        assert_eq!(
            config.database_acquire_timeout_secs,
            DEFAULT_ACQUIRE_TIMEOUT_SECS
        );
        assert_eq!(config.database_idle_timeout_secs, DEFAULT_IDLE_TIMEOUT_SECS);

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

    #[test]
    fn from_env_overrides_database_pool_settings() {
        let _guard = ENV_LOCK.lock().expect("mutex poisoned");

        clear_env();
        env::set_var("DATABASE_URL", "postgresql://localhost:5432/example_db");
        env::set_var("DATABASE_MAX_CONNECTIONS", "10");
        env::set_var("DATABASE_MIN_CONNECTIONS", "2");
        env::set_var("DATABASE_ACQUIRE_TIMEOUT_SECS", "15");
        env::set_var("DATABASE_IDLE_TIMEOUT_SECS", "120");

        let config = Config::from_env().expect("configuration should load");

        assert_eq!(config.database_max_connections, 10);
        assert_eq!(config.database_min_connections, 2);
        assert_eq!(config.database_acquire_timeout_secs, 15);
        assert_eq!(config.database_idle_timeout_secs, 120);

        clear_env();
    }

    #[test]
    fn from_env_fails_when_pool_setting_invalid() {
        let _guard = ENV_LOCK.lock().expect("mutex poisoned");

        clear_env();
        env::set_var("DATABASE_URL", "postgresql://localhost:5432/example_db");
        env::set_var("DATABASE_MAX_CONNECTIONS", "not-a-number");

        let error = Config::from_env().expect_err("invalid setting must error");

        assert!(matches!(
            error,
            ConfigError::InvalidNumber {
                name,
                ..
            } if name == "DATABASE_MAX_CONNECTIONS"
        ));

        clear_env();
    }
}
