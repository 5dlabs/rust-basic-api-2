use dotenv::dotenv;
use std::{env, io::ErrorKind, num::ParseIntError};
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
        if let Err(error) = dotenv() {
            if !matches!(
                error,
                dotenv::Error::Io(ref io_error) if io_error.kind() == ErrorKind::NotFound
            ) {
                tracing::debug!(?error, "Failed to load .env file");
            }
        }

        let database_url = match env::var("DATABASE_URL") {
            Ok(value) => value,
            Err(env::VarError::NotPresent) => {
                return Err(ConfigError::MissingEnv("DATABASE_URL".into()))
            }
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::InvalidUnicode("DATABASE_URL".into()))
            }
        };

        let server_port = parse_env_or_default("SERVER_PORT", 3000u16)?;

        Ok(Self {
            database_url,
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
    use crate::ENV_LOCK;
    use std::env;

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

    #[test]
    fn from_env_fails_when_port_invalid() {
        let _guard = ENV_LOCK.lock().expect("mutex poisoned");

        clear_env();
        env::set_var("DATABASE_URL", "postgresql://localhost:5432/example_db");
        env::set_var("SERVER_PORT", "not-a-number");

        let error = Config::from_env().expect_err("invalid port must error");

        assert!(matches!(
            error,
            ConfigError::InvalidNumber {
                name,
                ..
            } if name == "SERVER_PORT"
        ));

        clear_env();
    }
}
