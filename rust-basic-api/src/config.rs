use crate::error::ConfigError;
use dotenv::dotenv;
use std::env;

/// Application configuration derived from the process environment.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    /// Construct configuration by reading required environment variables.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url = match env::var("DATABASE_URL") {
            Ok(value) => value,
            Err(env::VarError::NotPresent) => {
                return Err(ConfigError::MissingEnvironment {
                    name: "DATABASE_URL",
                })
            }
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::InvalidUnicode {
                    name: "DATABASE_URL",
                })
            }
        };

        let server_port = match env::var("SERVER_PORT") {
            Ok(value) => value
                .parse::<u16>()
                .map_err(|source| ConfigError::InvalidPort { value, source })?,
            Err(env::VarError::NotPresent) => 3000,
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::InvalidUnicode {
                    name: "SERVER_PORT",
                })
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
    #[cfg(unix)]
    use std::ffi::OsString;
    #[cfg(unix)]
    use std::os::unix::ffi::OsStringExt;

    fn clear_env() {
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[cfg(unix)]
    fn set_non_unicode(var: &str) {
        let bytes = vec![0x66, 0x6f, 0x80];
        let value = OsString::from_vec(bytes);
        env::set_var(var, value);
    }

    #[test]
    #[serial]
    fn from_env_reads_values() {
        clear_env();
        env::set_var("DATABASE_URL", "postgres://postgres@localhost:5432/app_db");
        env::set_var("SERVER_PORT", "8080");

        let config = Config::from_env().expect("configuration should load");

        assert_eq!(
            config.database_url,
            "postgres://postgres@localhost:5432/app_db"
        );
        assert_eq!(config.server_port, 8080);
    }

    #[test]
    #[serial]
    fn from_env_reports_missing_database_url() {
        clear_env();
        env::set_var("SERVER_PORT", "3000");

        let error = Config::from_env().expect_err("Database URL is required");

        assert!(matches!(
            error,
            ConfigError::MissingEnvironment { name } if name == "DATABASE_URL"
        ));
    }

    #[test]
    #[serial]
    fn from_env_rejects_invalid_server_port() {
        clear_env();
        env::set_var("DATABASE_URL", "postgres://postgres@localhost:5432/app_db");
        env::set_var("SERVER_PORT", "not-a-number");

        let error = Config::from_env().expect_err("Invalid port value should error");

        assert!(matches!(error, ConfigError::InvalidPort { .. }));
    }

    #[test]
    #[serial]
    fn from_env_defaults_port_when_missing() {
        clear_env();
        env::set_var("DATABASE_URL", "postgres://postgres@localhost:5432/app_db");

        let config = Config::from_env().expect("default port should be provided");

        assert_eq!(config.server_port, 3000);
    }

    #[test]
    #[serial]
    #[cfg(unix)]
    fn from_env_rejects_non_unicode_database_url() {
        clear_env();
        set_non_unicode("DATABASE_URL");
        env::set_var("SERVER_PORT", "3000");

        let error = Config::from_env().expect_err("Invalid unicode should error");

        assert!(matches!(
            error,
            ConfigError::InvalidUnicode { name } if name == "DATABASE_URL"
        ));
    }

    #[test]
    #[serial]
    #[cfg(unix)]
    fn from_env_rejects_non_unicode_server_port() {
        clear_env();
        env::set_var("DATABASE_URL", "postgres://postgres@localhost:5432/app_db");
        set_non_unicode("SERVER_PORT");

        let error = Config::from_env().expect_err("Invalid unicode should error");

        assert!(matches!(
            error,
            ConfigError::InvalidUnicode { name } if name == "SERVER_PORT"
        ));
    }
}
