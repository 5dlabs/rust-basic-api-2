use std::env;

use dotenv::dotenv;

use crate::error::ConfigError;

/// Application configuration populated from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// The `DATABASE_URL` variable is required. `SERVER_PORT` is optional and
    /// defaults to `3000` when not provided.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url = read_env_var("DATABASE_URL")?;
        let server_port = match env::var("SERVER_PORT") {
            Ok(raw) => parse_port(raw)?,
            Err(env::VarError::NotPresent) => 3000,
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::InvalidUnicode {
                    name: "SERVER_PORT",
                });
            }
        };

        Ok(Self {
            database_url,
            server_port,
        })
    }
}

fn read_env_var(name: &'static str) -> Result<String, ConfigError> {
    match env::var(name) {
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => Err(ConfigError::MissingEnvVar { name }),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::InvalidUnicode { name }),
    }
}

fn parse_port(raw: String) -> Result<u16, ConfigError> {
    raw.parse::<u16>()
        .map_err(|source| ConfigError::InvalidServerPort { port: raw, source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_from_env_with_valid_config() {
        // Save and clear existing env vars first to avoid interference
        let existing_db = env::var("DATABASE_URL").ok();
        let existing_port = env::var("SERVER_PORT").ok();

        env::set_var("DATABASE_URL", "postgresql://localhost/testdb");
        env::set_var("SERVER_PORT", "8080");

        let config = Config::from_env().expect("Config should load successfully");

        assert_eq!(config.database_url, "postgresql://localhost/testdb");
        assert_eq!(config.server_port, 8080);

        // Restore original environment
        match existing_db {
            Some(val) => env::set_var("DATABASE_URL", val),
            None => env::remove_var("DATABASE_URL"),
        }
        match existing_port {
            Some(val) => env::set_var("SERVER_PORT", val),
            None => env::remove_var("SERVER_PORT"),
        }
    }

    #[test]
    #[serial]
    fn test_from_env_with_default_port() {
        let existing_db = env::var("DATABASE_URL").ok();
        let existing_port = env::var("SERVER_PORT").ok();

        env::set_var("DATABASE_URL", "postgresql://localhost/testdb");
        env::remove_var("SERVER_PORT");

        let config = Config::from_env().expect("Config should load with default port");

        assert_eq!(config.database_url, "postgresql://localhost/testdb");
        assert_eq!(config.server_port, 3000);

        match existing_db {
            Some(val) => env::set_var("DATABASE_URL", val),
            None => env::remove_var("DATABASE_URL"),
        }
        if let Some(val) = existing_port {
            env::set_var("SERVER_PORT", val);
        }
    }

    #[test]
    #[serial]
    fn test_from_env_missing_database_url() {
        env::remove_var("DATABASE_URL");

        let result = Config::from_env();

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::MissingEnvVar { name } => assert_eq!(name, "DATABASE_URL"),
            _ => panic!("Expected MissingEnvVar error"),
        }
    }

    #[test]
    #[serial]
    fn test_from_env_invalid_port_format() {
        let existing_db = env::var("DATABASE_URL").ok();
        let existing_port = env::var("SERVER_PORT").ok();

        env::set_var("DATABASE_URL", "postgresql://localhost/testdb");
        env::set_var("SERVER_PORT", "not_a_number");

        let result = Config::from_env();

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidServerPort { port, .. } => {
                assert_eq!(port, "not_a_number");
            }
            _ => panic!("Expected InvalidServerPort error"),
        }

        match existing_db {
            Some(val) => env::set_var("DATABASE_URL", val),
            None => env::remove_var("DATABASE_URL"),
        }
        match existing_port {
            Some(val) => env::set_var("SERVER_PORT", val),
            None => env::remove_var("SERVER_PORT"),
        }
    }

    #[test]
    #[serial]
    fn test_from_env_port_out_of_range() {
        let existing_db = env::var("DATABASE_URL").ok();
        let existing_port = env::var("SERVER_PORT").ok();

        env::set_var("DATABASE_URL", "postgresql://localhost/testdb");
        env::set_var("SERVER_PORT", "99999");

        let result = Config::from_env();

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidServerPort { .. } => {}
            _ => panic!("Expected InvalidServerPort error"),
        }

        match existing_db {
            Some(val) => env::set_var("DATABASE_URL", val),
            None => env::remove_var("DATABASE_URL"),
        }
        match existing_port {
            Some(val) => env::set_var("SERVER_PORT", val),
            None => env::remove_var("SERVER_PORT"),
        }
    }

    #[test]
    #[serial]
    fn test_from_env_negative_port() {
        let existing_db = env::var("DATABASE_URL").ok();
        let existing_port = env::var("SERVER_PORT").ok();

        env::set_var("DATABASE_URL", "postgresql://localhost/testdb");
        env::set_var("SERVER_PORT", "-1");

        let result = Config::from_env();

        assert!(result.is_err());

        match existing_db {
            Some(val) => env::set_var("DATABASE_URL", val),
            None => env::remove_var("DATABASE_URL"),
        }
        match existing_port {
            Some(val) => env::set_var("SERVER_PORT", val),
            None => env::remove_var("SERVER_PORT"),
        }
    }

    #[test]
    #[serial]
    fn test_from_env_zero_port() {
        let existing_db = env::var("DATABASE_URL").ok();
        let existing_port = env::var("SERVER_PORT").ok();

        env::set_var("DATABASE_URL", "postgresql://localhost/testdb");
        env::set_var("SERVER_PORT", "0");

        let config = Config::from_env().expect("Port 0 should be valid");

        assert_eq!(config.server_port, 0);

        match existing_db {
            Some(val) => env::set_var("DATABASE_URL", val),
            None => env::remove_var("DATABASE_URL"),
        }
        match existing_port {
            Some(val) => env::set_var("SERVER_PORT", val),
            None => env::remove_var("SERVER_PORT"),
        }
    }

    #[test]
    #[serial]
    fn test_from_env_max_port() {
        let existing_db = env::var("DATABASE_URL").ok();
        let existing_port = env::var("SERVER_PORT").ok();

        env::set_var("DATABASE_URL", "postgresql://localhost/testdb");
        env::set_var("SERVER_PORT", "65535");

        let config = Config::from_env().expect("Port 65535 should be valid");

        assert_eq!(config.server_port, 65535);

        match existing_db {
            Some(val) => env::set_var("DATABASE_URL", val),
            None => env::remove_var("DATABASE_URL"),
        }
        match existing_port {
            Some(val) => env::set_var("SERVER_PORT", val),
            None => env::remove_var("SERVER_PORT"),
        }
    }

    #[test]
    #[serial]
    fn test_read_env_var_success() {
        env::set_var("TEST_VAR", "test_value");

        let result = read_env_var("TEST_VAR");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_value");

        env::remove_var("TEST_VAR");
    }

    #[test]
    #[serial]
    fn test_read_env_var_missing() {
        env::remove_var("NONEXISTENT_VAR");

        let result = read_env_var("NONEXISTENT_VAR");

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::MissingEnvVar { name } => assert_eq!(name, "NONEXISTENT_VAR"),
            _ => panic!("Expected MissingEnvVar error"),
        }
    }

    #[test]
    #[serial]
    fn test_parse_port_valid() {
        let result = parse_port("8080".to_string());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 8080);
    }

    #[test]
    #[serial]
    fn test_parse_port_invalid() {
        let result = parse_port("invalid".to_string());

        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_config_clone() {
        let existing_db = env::var("DATABASE_URL").ok();

        env::set_var("DATABASE_URL", "postgresql://localhost/testdb");
        let config = Config::from_env().expect("Config should load");

        let cloned = config.clone();

        assert_eq!(config.database_url, cloned.database_url);
        assert_eq!(config.server_port, cloned.server_port);

        match existing_db {
            Some(val) => env::set_var("DATABASE_URL", val),
            None => env::remove_var("DATABASE_URL"),
        }
    }

    #[test]
    #[serial]
    fn test_config_debug() {
        let existing_db = env::var("DATABASE_URL").ok();

        env::set_var("DATABASE_URL", "postgresql://localhost/testdb");
        let config = Config::from_env().expect("Config should load");

        let debug_output = format!("{config:?}");

        assert!(debug_output.contains("Config"));
        assert!(debug_output.contains("database_url"));
        assert!(debug_output.contains("server_port"));

        match existing_db {
            Some(val) => env::set_var("DATABASE_URL", val),
            None => env::remove_var("DATABASE_URL"),
        }
    }
}
