//! Application configuration loaded from the environment.

use std::env;

use dotenv::dotenv;
use tracing::warn;

use crate::error::ConfigError;

const DEFAULT_SERVER_PORT: u16 = 3000;

/// Application configuration values sourced from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// `PostgreSQL` connection string used by the repository layer.
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    /// Load configuration from environment variables, falling back to sensible defaults where allowed.
    ///
    /// # Errors
    ///
    /// Returns a [`ConfigError`] when required environment variables are missing, cannot be parsed,
    /// or contain invalid unicode data.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();
        Self::from_env_with(|name| env::var(name))
    }

    /// Load configuration values using a custom environment variable provider.
    ///
    /// This helper enables deterministic testing by allowing callers to supply synthetic data
    /// instead of interacting with process-wide environment variables.
    ///
    /// # Errors
    ///
    /// Mirrors [`Config::from_env`] error semantics based on the provided data.
    pub fn from_env_with<F>(mut get_var: F) -> Result<Self, ConfigError>
    where
        F: FnMut(&str) -> Result<String, env::VarError>,
    {
        let database_url = match get_var("DATABASE_URL") {
            Ok(value) => value,
            Err(env::VarError::NotPresent) => {
                return Err(ConfigError::MissingEnvVar {
                    name: "DATABASE_URL",
                });
            }
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::InvalidUnicode {
                    name: "DATABASE_URL",
                });
            }
        };

        let server_port = match get_var("SERVER_PORT") {
            Ok(value) => match value.parse::<u16>() {
                Ok(port) => port,
                Err(source) => {
                    warn!(
                        %source,
                        default = DEFAULT_SERVER_PORT,
                        "Failed to parse SERVER_PORT; falling back to default port"
                    );
                    DEFAULT_SERVER_PORT
                }
            },
            Err(env::VarError::NotPresent) => DEFAULT_SERVER_PORT,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, env, env::VarError, ffi::OsString};

    use crate::test_support::env::guard as env_guard;

    #[derive(Clone, Copy)]
    enum MockVar {
        Present(&'static str),
        NotPresent,
        NotUnicode(&'static str),
    }

    fn mock_env_provider<'a>(
        values: &'a HashMap<&'static str, MockVar>,
    ) -> impl FnMut(&str) -> Result<String, VarError> + 'a {
        move |name| match values.get(name).copied().unwrap_or(MockVar::NotPresent) {
            MockVar::Present(value) => Ok(value.to_string()),
            MockVar::NotPresent => Err(VarError::NotPresent),
            MockVar::NotUnicode(value) => Err(VarError::NotUnicode(OsString::from(value))),
        }
    }

    #[test]
    fn loads_required_environment_variables() {
        let mut vars = HashMap::new();
        vars.insert(
            "DATABASE_URL",
            MockVar::Present("postgresql://localhost:5432/rust_basic_api"),
        );
        vars.insert("SERVER_PORT", MockVar::Present("8080"));

        let config = Config::from_env_with(mock_env_provider(&vars))
            .expect("config should load successfully");

        assert_eq!(
            config.database_url,
            "postgresql://localhost:5432/rust_basic_api"
        );
        assert_eq!(config.server_port, 8080);
    }

    #[test]
    fn defaults_server_port_to_3000_when_missing() {
        let mut vars = HashMap::new();
        vars.insert(
            "DATABASE_URL",
            MockVar::Present("postgresql://localhost:5432/rust_basic_api"),
        );

        let config = Config::from_env_with(mock_env_provider(&vars))
            .expect("config should load with defaults");

        assert_eq!(config.server_port, 3000);
    }

    #[test]
    fn missing_database_url_surfaces_error() {
        let mut vars = HashMap::new();
        vars.insert("SERVER_PORT", MockVar::Present("3000"));
        vars.insert("DATABASE_URL", MockVar::NotPresent);

        let result = Config::from_env_with(mock_env_provider(&vars));

        assert!(matches!(
            result,
            Err(ConfigError::MissingEnvVar { name }) if name == "DATABASE_URL"
        ));
    }

    #[test]
    fn invalid_server_port_falls_back_to_default() {
        let mut vars = HashMap::new();
        vars.insert(
            "DATABASE_URL",
            MockVar::Present("postgresql://localhost:5432/rust_basic_api"),
        );
        vars.insert("SERVER_PORT", MockVar::Present("not-a-number"));

        let config = Config::from_env_with(mock_env_provider(&vars))
            .expect("config should load with fallback default");

        assert_eq!(config.server_port, DEFAULT_SERVER_PORT);
    }

    #[test]
    fn invalid_unicode_database_url_surfaces_error() {
        let mut vars = HashMap::new();
        vars.insert("DATABASE_URL", MockVar::NotUnicode("invalid"));
        vars.insert("SERVER_PORT", MockVar::Present("3000"));

        let result = Config::from_env_with(mock_env_provider(&vars));

        assert!(matches!(
            result,
            Err(ConfigError::InvalidUnicode { name }) if name == "DATABASE_URL"
        ));
    }

    #[test]
    fn invalid_unicode_server_port_surfaces_error() {
        let mut vars = HashMap::new();
        vars.insert(
            "DATABASE_URL",
            MockVar::Present("postgresql://localhost:5432/rust_basic_api"),
        );
        vars.insert("SERVER_PORT", MockVar::NotUnicode("invalid"));

        let result = Config::from_env_with(mock_env_provider(&vars));

        assert!(matches!(
            result,
            Err(ConfigError::InvalidUnicode { name }) if name == "SERVER_PORT"
        ));
    }

    #[test]
    fn from_env_reads_process_environment() {
        let _guard = env_guard();
        unsafe {
            // SAFETY: the test serializes access to environment variables via `env_guard`.
            env::remove_var("DATABASE_URL");
            env::remove_var("SERVER_PORT");
        }

        unsafe {
            // SAFETY: the test serializes access to environment variables via `env_guard`.
            env::set_var("DATABASE_URL", "postgresql://localhost:5432/rust_basic_api");
            env::set_var("SERVER_PORT", "9090");
        }

        let config = Config::from_env().expect("configuration should load from environment");
        assert_eq!(
            config.database_url,
            "postgresql://localhost:5432/rust_basic_api"
        );
        assert_eq!(config.server_port, 9090);

        unsafe {
            // SAFETY: the test serializes access to environment variables via `env_guard`.
            env::remove_var("DATABASE_URL");
            env::remove_var("SERVER_PORT");
        }
    }
}
