use std::env::{self, VarError};

use dotenv::dotenv;
use thiserror::Error;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    /// Load configuration values from environment variables.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::EnvVar`] if the `DATABASE_URL` environment variable
    /// is not set or cannot be read.
    ///
    /// Returns [`ConfigError::InvalidPort`] if the `SERVER_PORT` environment variable
    /// is set but cannot be parsed as a valid `u16` port number.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").map_err(|source| ConfigError::EnvVar {
            var: "DATABASE_URL",
            source,
        })?;
        let server_port = match env::var("SERVER_PORT") {
            Ok(value) => match value.parse::<u16>() {
                Ok(port) => port,
                Err(source) => {
                    return Err(ConfigError::InvalidPort { value, source });
                }
            },
            Err(VarError::NotPresent) => 3000,
            Err(source) => {
                return Err(ConfigError::EnvVar {
                    var: "SERVER_PORT",
                    source,
                });
            }
        };

        Ok(Self {
            database_url,
            server_port,
        })
    }
}

/// Errors that can occur while loading configuration values.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read environment variable `{var}`: {source}")]
    EnvVar {
        var: &'static str,
        #[source]
        source: VarError,
    },

    #[error("invalid server port `{value}`")]
    InvalidPort {
        value: String,
        #[source]
        source: std::num::ParseIntError,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    const TEST_DATABASE_URL: &str = "postgresql://postgres@localhost:5432/rust_basic_api";

    fn reset_env() {
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn loads_configuration_from_environment() {
        reset_env();
        env::set_var("DATABASE_URL", TEST_DATABASE_URL);
        env::set_var("SERVER_PORT", "8080");

        let config = Config::from_env().expect("configuration should load");

        assert_eq!(config.database_url, TEST_DATABASE_URL);
        assert_eq!(config.server_port, 8080);

        reset_env();
    }

    #[test]
    #[serial]
    fn defaults_server_port_when_not_set() {
        reset_env();
        env::set_var("DATABASE_URL", TEST_DATABASE_URL);

        let config = Config::from_env().expect("configuration should load");

        assert_eq!(config.server_port, 3000);

        reset_env();
    }

    #[test]
    #[serial]
    fn missing_database_url_returns_error() {
        reset_env();

        let error = Config::from_env().expect_err("should fail without DATABASE_URL");

        assert!(matches!(
            error,
            ConfigError::EnvVar {
                var: "DATABASE_URL",
                ..
            }
        ));

        reset_env();
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn server_port_with_non_utf8_value_returns_error() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        reset_env();
        env::set_var("DATABASE_URL", TEST_DATABASE_URL);

        let invalid_bytes = OsStr::from_bytes(&[0x80]);
        env::set_var("SERVER_PORT", invalid_bytes);

        let error = Config::from_env().expect_err("invalid unicode should fail");

        assert!(matches!(
            error,
            ConfigError::EnvVar {
                var: "SERVER_PORT",
                ..
            }
        ));

        reset_env();
    }
}
