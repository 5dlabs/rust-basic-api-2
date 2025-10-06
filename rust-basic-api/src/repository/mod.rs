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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_read_env_u32_valid() {
        env::set_var("TEST_U32", "42");
        let result = read_env_u32("TEST_U32").unwrap();
        assert_eq!(result, Some(42));
        env::remove_var("TEST_U32");
    }

    #[test]
    #[serial]
    fn test_read_env_u32_missing() {
        env::remove_var("TEST_U32_MISSING");
        let result = read_env_u32("TEST_U32_MISSING").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    #[serial]
    fn test_read_env_u32_invalid() {
        env::set_var("TEST_U32_INVALID", "invalid");
        let result = read_env_u32("TEST_U32_INVALID");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("positive integer"));
        env::remove_var("TEST_U32_INVALID");
    }

    #[test]
    #[serial]
    fn test_read_env_u32_negative() {
        env::set_var("TEST_U32_NEG", "-5");
        let result = read_env_u32("TEST_U32_NEG");
        assert!(result.is_err());
        env::remove_var("TEST_U32_NEG");
    }

    #[test]
    #[serial]
    fn test_read_env_u32_zero() {
        env::set_var("TEST_U32_ZERO", "0");
        let result = read_env_u32("TEST_U32_ZERO").unwrap();
        assert_eq!(result, Some(0));
        env::remove_var("TEST_U32_ZERO");
    }

    #[test]
    #[serial]
    fn test_read_env_u64_valid() {
        env::set_var("TEST_U64", "123456");
        let result = read_env_u64("TEST_U64").unwrap();
        assert_eq!(result, Some(123456));
        env::remove_var("TEST_U64");
    }

    #[test]
    #[serial]
    fn test_read_env_u64_missing() {
        env::remove_var("TEST_U64_MISSING");
        let result = read_env_u64("TEST_U64_MISSING").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    #[serial]
    fn test_read_env_u64_invalid() {
        env::set_var("TEST_U64_INVALID", "not_a_number");
        let result = read_env_u64("TEST_U64_INVALID");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("positive integer"));
        env::remove_var("TEST_U64_INVALID");
    }

    #[test]
    #[serial]
    fn test_read_env_u64_zero() {
        env::set_var("TEST_U64_ZERO", "0");
        let result = read_env_u64("TEST_U64_ZERO").unwrap();
        assert_eq!(result, Some(0));
        env::remove_var("TEST_U64_ZERO");
    }

    #[test]
    #[serial]
    fn test_read_env_u64_large_value() {
        env::set_var("TEST_U64_LARGE", "18446744073709551615"); // u64::MAX
        let result = read_env_u64("TEST_U64_LARGE").unwrap();
        assert_eq!(result, Some(18446744073709551615));
        env::remove_var("TEST_U64_LARGE");
    }
}
