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
    ///
    /// # Errors
    ///
    /// Returns an error if `DATABASE_URL` is not set or if `SERVER_PORT` is set but invalid.
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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_config_from_env_with_valid_values() {
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
        env::set_var("SERVER_PORT", "8080");

        let config = Config::from_env().expect("Failed to load config");

        assert_eq!(
            config.database_url,
            "postgresql://test:test@localhost:5432/test"
        );
        assert_eq!(config.server_port, 8080);

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_from_env_default_port() {
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
        env::remove_var("SERVER_PORT");

        let config = Config::from_env().expect("Failed to load config");

        assert_eq!(
            config.database_url,
            "postgresql://test:test@localhost:5432/test"
        );
        assert_eq!(config.server_port, 3000);

        env::remove_var("DATABASE_URL");
    }

    #[test]
    #[serial]
    fn test_config_from_env_missing_database_url() {
        env::remove_var("DATABASE_URL");
        env::set_var("SERVER_PORT", "8080");

        let result = Config::from_env();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("DATABASE_URL"));

        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_from_env_invalid_port() {
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
        env::set_var("SERVER_PORT", "invalid");

        let result = Config::from_env();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("u16") || error_msg.contains("parse"),
            "Expected error about port parsing, got: {error_msg}"
        );

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_from_env_port_out_of_range() {
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
        env::set_var("SERVER_PORT", "99999");

        let result = Config::from_env();

        assert!(result.is_err());

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_from_env_port_zero() {
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
        env::set_var("SERVER_PORT", "0");

        let config = Config::from_env().expect("Failed to load config");

        assert_eq!(config.server_port, 0);

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_from_env_port_max_value() {
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
        env::set_var("SERVER_PORT", "65535");

        let config = Config::from_env().expect("Failed to load config");

        assert_eq!(config.server_port, 65535);

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_debug_impl() {
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
        env::set_var("SERVER_PORT", "3000");

        let config = Config::from_env().expect("Failed to load config");
        let debug_str = format!("{config:?}");

        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("database_url"));
        assert!(debug_str.contains("server_port"));

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_clone_impl() {
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
        env::set_var("SERVER_PORT", "3000");

        let config = Config::from_env().expect("Failed to load config");
        let cloned = config.clone();

        assert_eq!(config.database_url, cloned.database_url);
        assert_eq!(config.server_port, cloned.server_port);

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }
}
