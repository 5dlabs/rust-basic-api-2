//! Configuration management module
//!
//! Handles loading and validation of application configuration from environment variables.

use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    /// `PostgreSQL` database connection URL
    /// Note: Will be used in future tasks for database connections
    #[allow(dead_code)]
    pub database_url: String,
    /// HTTP server port
    pub server_port: u16,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are missing or invalid
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL")?;
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
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
    use std::sync::Mutex;

    // Mutex to ensure tests run serially
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_config_with_valid_env() {
        let _lock = TEST_MUTEX.lock().unwrap();

        env::set_var("DATABASE_URL", "postgresql://localhost:5432/test");
        env::set_var("SERVER_PORT", "8080");

        let config = Config::from_env().expect("Failed to load config");
        assert_eq!(config.database_url, "postgresql://localhost:5432/test");
        assert_eq!(config.server_port, 8080);

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_config_default_port() {
        let _lock = TEST_MUTEX.lock().unwrap();

        env::set_var("DATABASE_URL", "postgresql://localhost:5432/test");
        env::remove_var("SERVER_PORT");

        let config = Config::from_env().expect("Failed to load config");
        assert_eq!(config.server_port, 3000);

        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_config_missing_database_url() {
        let _lock = TEST_MUTEX.lock().unwrap();

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
        
        let result = Config::from_env();
        assert!(result.is_err());
    }
}
