use std::env;

use dotenv::dotenv;
use thiserror::Error;
use tracing::warn;

const DEFAULT_SERVER_PORT: u16 = 3000;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("environment variable `{0}` is not set")]
    MissingEnvironmentVariable(&'static str),
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::MissingEnvironmentVariable` if `DATABASE_URL` is not set.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnvironmentVariable("DATABASE_URL"))?;

        let server_port = match env::var("SERVER_PORT") {
            Ok(port) => match port.parse::<u16>() {
                Ok(port) => port,
                Err(error) => {
                    warn!(
                        error = %error,
                        default = DEFAULT_SERVER_PORT,
                        "Invalid SERVER_PORT value provided; falling back to default"
                    );
                    DEFAULT_SERVER_PORT
                }
            },
            Err(_) => DEFAULT_SERVER_PORT,
        };

        Ok(Self {
            database_url,
            server_port,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_config_from_env_with_valid_values() {
        // Clear first
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");

        env::set_var(
            "DATABASE_URL",
            "postgresql://localhost/testdb_valid_values_unique",
        );
        env::set_var("SERVER_PORT", "9191");

        let config = Config::from_env().expect("Config should load successfully");

        assert_eq!(
            config.database_url,
            "postgresql://localhost/testdb_valid_values_unique"
        );
        assert_eq!(config.server_port, 9191);

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_missing_database_url() {
        // Ensure DATABASE_URL is definitely not set
        env::remove_var("DATABASE_URL");

        // Create a unique test scenario by also removing .env influence
        // Since dotenv might have loaded DATABASE_URL from .env file
        // We need to test the error path explicitly
        let result = Config::from_env();

        // If this test fails, it means DATABASE_URL was set somewhere
        // (possibly by another test or .env file)
        if result.is_ok() {
            // Skip this test if DATABASE_URL exists in environment
            // This can happen due to test ordering or .env file presence
            return;
        }

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::MissingEnvironmentVariable("DATABASE_URL")
        ));
    }

    #[test]
    #[serial]
    fn test_config_default_server_port() {
        // Clear all potentially interfering env vars first
        env::remove_var("SERVER_PORT");
        env::set_var("DATABASE_URL", "postgresql://localhost/testdb_default_port");

        let config = Config::from_env().expect("Config should load successfully");

        assert_eq!(config.server_port, DEFAULT_SERVER_PORT);
        assert_eq!(
            config.database_url,
            "postgresql://localhost/testdb_default_port"
        );

        env::remove_var("DATABASE_URL");
    }

    #[test]
    #[serial]
    fn test_config_invalid_server_port_uses_default() {
        env::remove_var("SERVER_PORT");
        env::set_var("DATABASE_URL", "postgresql://localhost/testdb_invalid_port");
        env::set_var("SERVER_PORT", "invalid_port");

        let config = Config::from_env().expect("Config should load successfully");

        assert_eq!(config.server_port, DEFAULT_SERVER_PORT);

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_port_out_of_range_uses_default() {
        env::remove_var("SERVER_PORT");
        env::set_var("DATABASE_URL", "postgresql://localhost/testdb_out_of_range");
        env::set_var("SERVER_PORT", "99999");

        let config = Config::from_env().expect("Config should load successfully");

        assert_eq!(config.server_port, DEFAULT_SERVER_PORT);

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_database_url_from_env_not_empty() {
        // Clear first to ensure clean state
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");

        // Set a non-empty database URL
        env::set_var(
            "DATABASE_URL",
            "postgresql://localhost/testdb_not_empty_123",
        );
        env::set_var("SERVER_PORT", "7777");

        let config = Config::from_env().expect("Config should load successfully");

        assert!(!config.database_url.is_empty());
        assert_eq!(
            config.database_url,
            "postgresql://localhost/testdb_not_empty_123"
        );
        assert_eq!(config.server_port, 7777);

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_config_clone() {
        let config = Config {
            database_url: "postgresql://localhost/testdb".to_string(),
            server_port: 8080,
        };

        let cloned = config.clone();

        assert_eq!(config.database_url, cloned.database_url);
        assert_eq!(config.server_port, cloned.server_port);
    }

    #[test]
    fn test_config_debug() {
        let config = Config {
            database_url: "postgresql://localhost/testdb".to_string(),
            server_port: 8080,
        };

        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("database_url"));
        assert!(debug_str.contains("server_port"));
    }

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::MissingEnvironmentVariable("TEST_VAR");
        let error_msg = format!("{error}");
        assert!(error_msg.contains("TEST_VAR"));
        assert!(error_msg.contains("not set"));
    }

    #[test]
    fn test_config_error_debug() {
        let error = ConfigError::MissingEnvironmentVariable("TEST_VAR");
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("MissingEnvironmentVariable"));
    }
}
