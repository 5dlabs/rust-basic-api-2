use anyhow::{Context, Result};
use dotenv::dotenv;
use std::env;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    /// Load configuration values from the current environment.
    ///
    /// Falls back to sensible defaults where appropriate and returns an error
    /// if required values are missing or malformed.
    pub fn from_env() -> Result<Self> {
        dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").context("DATABASE_URL environment variable is required")?;

        let server_port = env::var("SERVER_PORT")
            .ok()
            .map(|value| {
                value
                    .parse::<u16>()
                    .context("SERVER_PORT must be a valid unsigned 16-bit integer")
            })
            .transpose()? // propagate parsing error if present
            .unwrap_or(3000);

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

    #[test]
    #[serial]
    fn test_config_with_valid_database_url() {
        env::set_var(
            "DATABASE_URL",
            "postgresql://test:test@localhost:5432/testdb",
        );
        env::set_var("SERVER_PORT", "8080");

        let config = Config::from_env();
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(
            config.database_url,
            "postgresql://test:test@localhost:5432/testdb"
        );
        assert_eq!(config.server_port, 8080);

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_defaults_to_port_3000() {
        env::set_var(
            "DATABASE_URL",
            "postgresql://test:test@localhost:5432/testdb",
        );
        env::remove_var("SERVER_PORT");

        let config = Config::from_env();
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.server_port, 3000);

        env::remove_var("DATABASE_URL");
    }

    #[test]
    #[serial]
    fn test_config_missing_database_url_fails() {
        env::remove_var("DATABASE_URL");
        env::set_var("SERVER_PORT", "8080");

        let config = Config::from_env();
        assert!(config.is_err());

        let err_msg = config.unwrap_err().to_string();
        assert!(err_msg.contains("DATABASE_URL"));

        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_invalid_port_fails() {
        env::set_var(
            "DATABASE_URL",
            "postgresql://test:test@localhost:5432/testdb",
        );
        env::set_var("SERVER_PORT", "invalid");

        let config = Config::from_env();
        assert!(config.is_err());

        let err_msg = config.unwrap_err().to_string();
        assert!(err_msg.contains("SERVER_PORT"));

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_port_out_of_range_fails() {
        env::set_var(
            "DATABASE_URL",
            "postgresql://test:test@localhost:5432/testdb",
        );
        env::set_var("SERVER_PORT", "70000");

        let config = Config::from_env();
        assert!(config.is_err());

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_clone() {
        env::set_var(
            "DATABASE_URL",
            "postgresql://test:test@localhost:5432/testdb",
        );
        env::set_var("SERVER_PORT", "8080");

        let config = Config::from_env().unwrap();
        let cloned = config.clone();

        assert_eq!(config.database_url, cloned.database_url);
        assert_eq!(config.server_port, cloned.server_port);

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_debug_impl() {
        env::set_var(
            "DATABASE_URL",
            "postgresql://test:test@localhost:5432/testdb",
        );

        let config = Config::from_env().unwrap();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("database_url"));
        assert!(debug_str.contains("server_port"));

        env::remove_var("DATABASE_URL");
    }
}
