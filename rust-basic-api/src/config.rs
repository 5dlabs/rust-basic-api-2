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
