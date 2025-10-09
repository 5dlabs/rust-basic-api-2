use crate::error::AppError;
use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").map_err(|_| {
            AppError::Configuration("DATABASE_URL environment variable is not set".into())
        })?;

        let server_port = match env::var("SERVER_PORT") {
            Ok(port) => port.parse::<u16>().map_err(|error| {
                AppError::Configuration(format!(
                    "SERVER_PORT must be a valid u16, got `{port}`: {error}"
                ))
            })?,
            Err(env::VarError::NotPresent) => {
                tracing::debug!("SERVER_PORT not set; defaulting to 3000");
                3000
            }
            Err(env::VarError::NotUnicode(_)) => {
                return Err(AppError::Configuration(
                    "SERVER_PORT contains invalid unicode characters".into(),
                ));
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

    fn clear_env() {
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    fn example_database_url() -> String {
        format!(
            "{scheme}://{user}:{password}@{host}:{port}/{database}",
            scheme = "postgres",
            user = "example_user",
            password = "example_secret",
            host = "localhost",
            port = 5432,
            database = "example_db"
        )
    }

    #[test]
    #[serial]
    fn from_env_loads_expected_values() {
        clear_env();

        let expected_database_url = example_database_url();
        env::set_var("DATABASE_URL", &expected_database_url);
        env::set_var("SERVER_PORT", "4123");

        let config = Config::from_env().expect("config should load from environment");
        assert_eq!(config.database_url, expected_database_url);
        assert_eq!(config.server_port, 4123);

        clear_env();
    }

    #[test]
    #[serial]
    fn from_env_defaults_port_when_missing() {
        clear_env();

        env::set_var("DATABASE_URL", example_database_url());

        let config = Config::from_env().expect("config should load with default port");
        assert_eq!(config.server_port, 3000);

        clear_env();
    }

    #[test]
    #[serial]
    fn from_env_errors_when_database_url_missing() {
        clear_env();

        env::set_var("SERVER_PORT", "3000");

        let error = Config::from_env().expect_err("missing DATABASE_URL must error");

        match error {
            AppError::Configuration(message) => {
                assert!(message.contains("DATABASE_URL"));
            }
            other => panic!("expected configuration error, got {other:?}"),
        }

        clear_env();
    }

    #[test]
    #[serial]
    fn from_env_errors_on_invalid_port() {
        clear_env();

        env::set_var("DATABASE_URL", example_database_url());
        env::set_var("SERVER_PORT", "not-a-number");

        let error = Config::from_env().expect_err("invalid port must error");

        match error {
            AppError::Configuration(message) => {
                assert!(message.contains("SERVER_PORT"));
            }
            other => panic!("expected configuration error, got {other:?}"),
        }

        clear_env();
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn from_env_errors_on_non_unicode_port() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        clear_env();

        env::set_var("DATABASE_URL", example_database_url());
        env::set_var("SERVER_PORT", OsString::from_vec(vec![0xFF]));

        let error = Config::from_env().expect_err("non-unicode port must error");

        match error {
            AppError::Configuration(message) => {
                assert!(message.contains("invalid unicode"));
            }
            other => panic!("expected configuration error, got {other:?}"),
        }

        clear_env();
    }
}
