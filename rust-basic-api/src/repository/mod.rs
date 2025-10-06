//! Database connectivity and repository utilities.

use std::{env, time::Duration};

use anyhow::{anyhow, Context, Result};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::config::Config;

/// Thin wrapper around the `SQLx` `PostgreSQL` connection pool.
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Initialise a `PostgreSQL` connection pool using the provided configuration.
    pub fn connect(config: &Config) -> Result<Self> {
        let max_connections = read_env_u32("DATABASE_MAX_CONNECTIONS")?.unwrap_or(5);
        let acquire_timeout_secs = read_env_u64("DATABASE_ACQUIRE_TIMEOUT_SECS")?.unwrap_or(5);

        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(Duration::from_secs(acquire_timeout_secs))
            .connect_lazy(&config.database_url)
            .context("failed to create PostgreSQL connection pool")?;

        Ok(Self { pool })
    }

    /// Execute a lightweight query to verify database connectivity.
    pub async fn is_healthy(&self) -> std::result::Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
    }
}

fn read_env_u32(key: &str) -> Result<Option<u32>> {
    match env::var(key) {
        Ok(value) => {
            let parsed = value
                .parse::<u32>()
                .with_context(|| format!("{key} must be a positive integer"))?;
            Ok(Some(parsed))
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => {
            Err(anyhow!("{key} contains invalid UTF-8 characters"))
        }
    }
}

fn read_env_u64(key: &str) -> Result<Option<u64>> {
    match env::var(key) {
        Ok(value) => {
            let parsed = value
                .parse::<u64>()
                .with_context(|| format!("{key} must be a positive integer"))?;
            Ok(Some(parsed))
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => {
            Err(anyhow!("{key} contains invalid UTF-8 characters"))
        }
    }
}
