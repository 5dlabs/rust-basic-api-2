use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let server_port = std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()?;

        Ok(Config { server_port })
    }
}
