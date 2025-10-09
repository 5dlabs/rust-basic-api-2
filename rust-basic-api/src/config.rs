use crate::error::ConfigError;
use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub database_max_connections: u32,
}

impl Config {
    /// Loads configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if required environment variables are missing,
    /// contain invalid values, or cannot be parsed.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url = require_env("DATABASE_URL")?;
        let server_port = optional_port("SERVER_PORT")?;
        let database_max_connections = optional_nonzero_u32("DATABASE_MAX_CONNECTIONS", 5)?;

        Ok(Self {
            database_url,
            server_port,
            database_max_connections,
        })
    }

    #[must_use]
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    #[must_use]
    pub fn server_port(&self) -> u16 {
        self.server_port
    }

    #[must_use]
    pub fn database_max_connections(&self) -> u32 {
        self.database_max_connections
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

fn optional_nonzero_u32(key: &str, default: u32) -> Result<u32, ConfigError> {
    match env::var(key) {
        Ok(value) => {
            if value.trim().is_empty() {
                return Err(ConfigError::EmptyEnv {
                    key: key.to_string(),
                });
            }

            let parsed = value
                .parse::<u32>()
                .map_err(|source| ConfigError::InvalidValue {
                    key: key.to_string(),
                    source,
                })?;

            if parsed == 0 {
                return Err(ConfigError::InvalidRange {
                    key: key.to_string(),
                    min: 1,
                });
            }

            Ok(parsed)
        }
        Err(env::VarError::NotPresent) => Ok(default),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::InvalidUnicode {
            key: key.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_config_with_required_vars() {
        env::set_var("DATABASE_URL", "postgresql://localhost:5432/testdb");
        env::set_var("SERVER_PORT", "8080");
        env::set_var("DATABASE_MAX_CONNECTIONS", "10");

        let config = Config::from_env().unwrap();

        assert_eq!(config.database_url(), "postgresql://localhost:5432/testdb");
        assert_eq!(config.server_port(), 8080);
        assert_eq!(config.database_max_connections(), 10);

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
        env::remove_var("DATABASE_MAX_CONNECTIONS");
    }

    #[test]
    #[serial]
    fn test_config_with_defaults() {
        env::set_var("DATABASE_URL", "postgresql://localhost:5432/testdb");
        env::remove_var("SERVER_PORT");
        env::remove_var("DATABASE_MAX_CONNECTIONS");

        let config = Config::from_env().unwrap();

        assert_eq!(config.server_port(), 3000);
        assert_eq!(config.database_max_connections(), 5);

        env::remove_var("DATABASE_URL");
    }

    #[test]
    #[serial]
    fn test_config_missing_database_url() {
        env::remove_var("DATABASE_URL");

        let result = Config::from_env();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::MissingEnv { .. }
        ));
    }

    #[test]
    #[serial]
    fn test_config_invalid_port() {
        env::set_var("DATABASE_URL", "postgresql://localhost:5432/testdb");
        env::set_var("SERVER_PORT", "invalid");

        let result = Config::from_env();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::InvalidValue { .. }
        ));

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_empty_database_url() {
        // Clean up any leftover vars from other tests
        env::remove_var("SERVER_PORT");
        env::remove_var("DATABASE_MAX_CONNECTIONS");
        env::set_var("DATABASE_URL", "");

        let result = Config::from_env();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::EmptyEnv { .. }));

        env::remove_var("DATABASE_URL");
    }

    #[test]
    #[serial]
    fn test_config_zero_max_connections() {
        env::set_var("DATABASE_URL", "postgresql://localhost:5432/testdb");
        env::set_var("DATABASE_MAX_CONNECTIONS", "0");

        let result = Config::from_env();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::InvalidRange { .. }
        ));

        env::remove_var("DATABASE_URL");
        env::remove_var("DATABASE_MAX_CONNECTIONS");
    }
}
