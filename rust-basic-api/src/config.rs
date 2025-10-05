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
    ///
    /// # Errors
    ///
    /// Returns a [`ConfigError`] if:
    /// - `DATABASE_URL` environment variable is not set
    /// - `SERVER_HOST` contains an invalid IP address
    /// - `SERVER_PORT` contains an invalid port number
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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn setup_env(db_url: &str, host: Option<&str>, port: Option<&str>) {
        env::set_var("DATABASE_URL", db_url);
        if let Some(h) = host {
            env::set_var("SERVER_HOST", h);
        } else {
            env::remove_var("SERVER_HOST");
        }
        if let Some(p) = port {
            env::set_var("SERVER_PORT", p);
        } else {
            env::remove_var("SERVER_PORT");
        }
    }

    fn cleanup_env() {
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_HOST");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_with_all_values() {
        setup_env(
            "postgresql://test:test@localhost:5432/db",
            Some("127.0.0.1"),
            Some("8080"),
        );

        let config = Config::from_env().expect("Config should load");

        assert_eq!(
            config.database_url,
            "postgresql://test:test@localhost:5432/db"
        );
        assert_eq!(config.server_host.to_string(), "127.0.0.1");
        assert_eq!(config.server_port, 8080);

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_with_defaults() {
        setup_env("postgresql://test:test@localhost:5432/db", None, None);

        let config = Config::from_env().expect("Config should load");

        assert_eq!(
            config.database_url,
            "postgresql://test:test@localhost:5432/db"
        );
        assert_eq!(config.server_host.to_string(), "0.0.0.0");
        assert_eq!(config.server_port, 3000);

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_missing_database_url() {
        cleanup_env();

        let result = Config::from_env();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::Missing("DATABASE_URL")));

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_invalid_host() {
        setup_env(
            "postgresql://test:test@localhost:5432/db",
            Some("invalid-host"),
            Some("3000"),
        );

        let result = Config::from_env();

        assert!(result.is_err());
        if let Err(ConfigError::InvalidEnvVar { field, .. }) = result {
            assert_eq!(field, "SERVER_HOST");
        } else {
            panic!("Expected InvalidEnvVar error");
        }

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_invalid_port() {
        setup_env(
            "postgresql://test:test@localhost:5432/db",
            None,
            Some("invalid"),
        );

        let result = Config::from_env();

        assert!(result.is_err());
        if let Err(ConfigError::InvalidEnvVar { field, .. }) = result {
            assert_eq!(field, "SERVER_PORT");
        } else {
            panic!("Expected InvalidEnvVar error");
        }

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_port_out_of_range() {
        setup_env(
            "postgresql://test:test@localhost:5432/db",
            None,
            Some("70000"),
        );

        let result = Config::from_env();

        assert!(result.is_err());

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_clone() {
        setup_env(
            "postgresql://test:test@localhost:5432/db",
            Some("192.168.1.1"),
            Some("8888"),
        );

        let config = Config::from_env().expect("Config should load");
        let cloned = config.clone();

        assert_eq!(config.database_url, cloned.database_url);
        assert_eq!(config.server_host, cloned.server_host);
        assert_eq!(config.server_port, cloned.server_port);

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_debug() {
        setup_env("postgresql://test:test@localhost:5432/db", None, None);

        let config = Config::from_env().expect("Config should load");
        let debug_str = format!("{config:?}");

        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("database_url"));
        assert!(debug_str.contains("server_host"));
        assert!(debug_str.contains("server_port"));

        cleanup_env();
    }
}
