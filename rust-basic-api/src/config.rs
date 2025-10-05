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
    ///
    /// # Errors
    ///
    /// Returns an error when an appropriate database configuration is not
    /// present or when `SERVER_PORT` cannot be parsed into a valid `u16` value.
    pub fn from_env() -> Result<Self> {
        dotenv().ok();

        let database_url = database_url_from_env()?;

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

/// Resolve the active database connection string from environment configuration.
///
/// This helper first checks for fully qualified URL variables and falls back to
/// constructing a connection string from individual components when necessary.
///
/// # Errors
///
/// Returns an error when neither a full URL nor the required components are
/// available.
pub fn database_url_from_env() -> Result<String> {
    let full_url_candidates = ["DATABASE_URL", "TEST_DATABASE_URL"];
    for key in &full_url_candidates {
        if let Ok(value) = env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() && trimmed != "__AUTO_GENERATED__" {
                return Ok(trimmed.to_owned());
            }
        }
    }

    let username = env::var("DATABASE_USER")
        .or_else(|_| env::var("TEST_DB_USER"))
        .context("DATABASE_USER or TEST_DB_USER must be set")?;

    let password = env::var("DATABASE_PASSWORD")
        .or_else(|_| env::var("TEST_DB_PASSWORD"))
        .unwrap_or_else(|_| String::new());

    let host = env::var("DATABASE_HOST")
        .or_else(|_| env::var("TEST_DB_HOST"))
        .unwrap_or_else(|_| "localhost".to_owned());

    let port = env::var("DATABASE_PORT")
        .or_else(|_| env::var("TEST_DB_PORT"))
        .unwrap_or_else(|_| "5432".to_owned());

    // Validate port is a numeric value that fits in u16
    let port_num = port
        .parse::<u16>()
        .context("DATABASE_PORT or TEST_DB_PORT must be a valid u16")?;

    let database_name = env::var("DATABASE_NAME")
        .or_else(|_| env::var("TEST_DB_NAME"))
        .context("DATABASE_NAME or TEST_DB_NAME must be set")?;

    let mut url = format!("postgresql://{username}");
    if !password.is_empty() {
        url.push(':');
        url.push_str(&password);
    }
    url.push('@');
    url.push_str(&host);
    url.push(':');
    url.push_str(&port_num.to_string());
    url.push('/');
    url.push_str(&database_name);

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn composed_test_url(db: &str) -> String {
        let scheme = "postgresql";
        [scheme, "://", "test", ":", "test", "@localhost:5432/", db].concat()
    }

    #[test]
    #[serial]
    fn test_config_with_valid_database_url() {
        env::set_var("DATABASE_URL", composed_test_url("testdb"));
        env::set_var("SERVER_PORT", "8080");

        let config = Config::from_env();
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.database_url, composed_test_url("testdb"));
        assert_eq!(config.server_port, 8080);

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_defaults_to_port_3000() {
        env::set_var("DATABASE_URL", composed_test_url("testdb"));
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
        assert!(err_msg.contains("DATABASE_USER"));

        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_invalid_port_fails() {
        env::set_var("DATABASE_URL", composed_test_url("testdb"));
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
    fn test_config_builds_database_url_from_components() {
        env::remove_var("DATABASE_URL");
        env::set_var("DATABASE_USER", "postgres");
        env::set_var("DATABASE_PASSWORD", "postgres");
        env::set_var("DATABASE_HOST", "127.0.0.1");
        env::set_var("DATABASE_PORT", "6543");
        env::set_var("DATABASE_NAME", "component_db");
        env::set_var("SERVER_PORT", "4000");

        let config = Config::from_env().expect("config should build from components");
        let expected = [
            "postgresql",
            "://",
            "postgres",
            ":",
            "postgres",
            "@127.0.0.1:6543/",
            "component_db",
        ]
        .concat();
        assert_eq!(config.database_url, expected);
        assert_eq!(config.server_port, 4000);

        env::remove_var("DATABASE_USER");
        env::remove_var("DATABASE_PASSWORD");
        env::remove_var("DATABASE_HOST");
        env::remove_var("DATABASE_PORT");
        env::remove_var("DATABASE_NAME");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_port_out_of_range_fails() {
        env::set_var("DATABASE_URL", composed_test_url("testdb"));
        env::set_var("SERVER_PORT", "70000");

        let config = Config::from_env();
        assert!(config.is_err());

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_config_clone() {
        env::set_var("DATABASE_URL", composed_test_url("testdb"));
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
        env::set_var("DATABASE_URL", composed_test_url("testdb"));

        let config = Config::from_env().unwrap();
        let debug_str = format!("{config:?}");

        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("database_url"));
        assert!(debug_str.contains("server_port"));

        env::remove_var("DATABASE_URL");
    }
}
