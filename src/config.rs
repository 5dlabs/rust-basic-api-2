use std::env;

use dotenv::dotenv;
use thiserror::Error;
use tracing::warn;

const DEFAULT_SERVER_PORT: u16 = 3000;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("environment variable `{0}` is not set")]
    MissingEnvironmentVariable(&'static str),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnvironmentVariable("DATABASE_URL"))?;

        let server_port = match env::var("SERVER_PORT") {
            Ok(port) => match port.parse::<u16>() {
                Ok(port) => port,
                Err(error) => {
                    warn!(
                        error = %error,
                        default = DEFAULT_SERVER_PORT,
                        "Invalid SERVER_PORT value provided; falling back to default"
                    );
                    DEFAULT_SERVER_PORT
                }
            },
            Err(_) => DEFAULT_SERVER_PORT,
        };

        Ok(Self {
            database_url,
            server_port,
        })
    }
}
