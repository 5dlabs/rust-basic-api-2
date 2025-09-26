use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    #[allow(dead_code)] // Used in future tasks
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")?;
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);

        Ok(Config {
            database_url,
            server_port,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_from_env_with_all_vars() {
        // Set up test environment variables
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");
        env::set_var("SERVER_PORT", "8080");

        let config = Config::from_env().expect("Config should be created successfully");

        assert_eq!(config.database_url, "postgresql://test:test@localhost:5432/testdb");
        assert_eq!(config.server_port, 8080);

        // Clean up
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_config_from_env_missing_database_url() {
        // Ensure DATABASE_URL is not set
        env::remove_var("DATABASE_URL");
        env::set_var("SERVER_PORT", "8080");

        let result = Config::from_env();

        assert!(result.is_err(), "Config should fail when DATABASE_URL is missing");

        // Clean up
        env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_config_from_env_default_port() {
        // Set up test environment with only DATABASE_URL
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");
        env::remove_var("SERVER_PORT");

        let config = Config::from_env().expect("Config should be created successfully");

        assert_eq!(config.database_url, "postgresql://test:test@localhost:5432/testdb");
        assert_eq!(config.server_port, 3000, "Should use default port 3000");

        // Clean up
        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_config_from_env_invalid_port_fallback() {
        // Set up test environment with invalid port
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");
        env::set_var("SERVER_PORT", "invalid_port");

        let config = Config::from_env().expect("Config should be created successfully");

        assert_eq!(config.database_url, "postgresql://test:test@localhost:5432/testdb");
        assert_eq!(config.server_port, 3000, "Should fallback to default port 3000");

        // Clean up
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_config_from_env_zero_port_fallback() {
        // Set up test environment with zero port
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");
        env::set_var("SERVER_PORT", "0");

        let config = Config::from_env().expect("Config should be created successfully");

        assert_eq!(config.database_url, "postgresql://test:test@localhost:5432/testdb");
        assert_eq!(config.server_port, 0, "Should accept port 0");

        // Clean up
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_config_from_env_max_port() {
        // Set up test environment with maximum port
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");
        env::set_var("SERVER_PORT", "65535");

        let config = Config::from_env().expect("Config should be created successfully");

        assert_eq!(config.database_url, "postgresql://test:test@localhost:5432/testdb");
        assert_eq!(config.server_port, 65535, "Should accept maximum port 65535");

        // Clean up
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_config_from_env_negative_port_fallback() {
        // Set up test environment with negative port (this would actually be parsed as invalid)
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");
        env::set_var("SERVER_PORT", "-1");

        let config = Config::from_env().expect("Config should be created successfully");

        assert_eq!(config.database_url, "postgresql://test:test@localhost:5432/testdb");
        assert_eq!(config.server_port, 3000, "Should fallback to default port 3000 for negative values");

        // Clean up
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_config_from_env_over_max_port_fallback() {
        // Set up test environment with port over maximum u16 value
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");
        env::set_var("SERVER_PORT", "65536");

        let config = Config::from_env().expect("Config should be created successfully");

        assert_eq!(config.database_url, "postgresql://test:test@localhost:5432/testdb");
        assert_eq!(config.server_port, 3000, "Should fallback to default port 3000 for over-max values");

        // Clean up
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_config_debug_impl() {
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");
        env::set_var("SERVER_PORT", "8080");

        let config = Config::from_env().expect("Config should be created successfully");
        let debug_string = format!("{:?}", config);

        assert!(debug_string.contains("Config"));
        assert!(debug_string.contains("database_url"));
        assert!(debug_string.contains("server_port"));
        assert!(debug_string.contains("8080"));

        // Clean up
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_config_clone_impl() {
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");
        env::set_var("SERVER_PORT", "8080");

        let config = Config::from_env().expect("Config should be created successfully");
        let config_clone = config.clone();

        assert_eq!(config.database_url, config_clone.database_url);
        assert_eq!(config.server_port, config_clone.server_port);

        // Clean up
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }
}