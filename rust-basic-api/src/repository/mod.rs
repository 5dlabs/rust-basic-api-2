//! Database connectivity and repository utilities.

use std::{env, time::Duration};

use anyhow::{anyhow, bail, Context, Result};
use sqlx::{postgres::PgPoolOptions, PgPool};

const DEFAULT_MAX_CONNECTIONS: u32 = 10;
const DEFAULT_MIN_CONNECTIONS: u32 = 2;
const DEFAULT_ACQUIRE_TIMEOUT_SECS: u64 = 3;
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 600;
const DEFAULT_MAX_LIFETIME_SECS: u64 = 1_800;

/// Create a new asynchronous `PostgreSQL` connection pool using `SQLx`.
///
/// # Errors
///
/// Returns an error if the pool configuration is invalid or the database connection fails.
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let settings = PoolSettings::from_env()?;

    let mut options = PgPoolOptions::new()
        .max_connections(settings.max_connections)
        .min_connections(settings.min_connections)
        .acquire_timeout(Duration::from_secs(settings.acquire_timeout_secs));

    if let Some(idle_timeout) = settings.idle_timeout_secs {
        options = options.idle_timeout(Duration::from_secs(idle_timeout));
    }

    if let Some(max_lifetime) = settings.max_lifetime_secs {
        options = options.max_lifetime(Duration::from_secs(max_lifetime));
    }

    options
        .connect(database_url)
        .await
        .with_context(|| "failed to create PostgreSQL connection pool".to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PoolSettings {
    max_connections: u32,
    min_connections: u32,
    acquire_timeout_secs: u64,
    idle_timeout_secs: Option<u64>,
    max_lifetime_secs: Option<u64>,
}

impl PoolSettings {
    fn from_env() -> Result<Self> {
        let max_connections =
            read_env_u32("DATABASE_MAX_CONNECTIONS")?.unwrap_or(DEFAULT_MAX_CONNECTIONS);
        let min_connections =
            read_env_u32("DATABASE_MIN_CONNECTIONS")?.unwrap_or(DEFAULT_MIN_CONNECTIONS);

        if min_connections > max_connections {
            bail!("DATABASE_MIN_CONNECTIONS cannot exceed DATABASE_MAX_CONNECTIONS");
        }

        let acquire_timeout_secs =
            read_env_u64("DATABASE_ACQUIRE_TIMEOUT_SECS")?.unwrap_or(DEFAULT_ACQUIRE_TIMEOUT_SECS);

        let idle_timeout_secs = match read_env_u64("DATABASE_IDLE_TIMEOUT_SECS")? {
            Some(0) => None,
            Some(value) => Some(value),
            None => Some(DEFAULT_IDLE_TIMEOUT_SECS),
        };

        let max_lifetime_secs = match read_env_u64("DATABASE_MAX_LIFETIME_SECS")? {
            Some(0) => None,
            Some(value) => Some(value),
            None => Some(DEFAULT_MAX_LIFETIME_SECS),
        };

        Ok(Self {
            max_connections,
            min_connections,
            acquire_timeout_secs,
            idle_timeout_secs,
            max_lifetime_secs,
        })
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

pub mod test_utils;

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
    #[cfg(unix)]
    fn test_read_env_u32_not_unicode() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let os_value = OsString::from_vec(vec![0x80]);
        env::set_var("TEST_U32_NOT_UNICODE", os_value);

        let result = read_env_u32("TEST_U32_NOT_UNICODE");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid UTF-8"));

        env::remove_var("TEST_U32_NOT_UNICODE");
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
        assert_eq!(result, Some(123_456));
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
    #[cfg(unix)]
    fn test_read_env_u64_not_unicode() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let os_value = OsString::from_vec(vec![0x80]);
        env::set_var("TEST_U64_NOT_UNICODE", os_value);

        let result = read_env_u64("TEST_U64_NOT_UNICODE");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid UTF-8"));

        env::remove_var("TEST_U64_NOT_UNICODE");
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
        env::set_var("TEST_U64_LARGE", "18446744073709551615");
        let result = read_env_u64("TEST_U64_LARGE").unwrap();
        assert_eq!(result, Some(18_446_744_073_709_551_615));
        env::remove_var("TEST_U64_LARGE");
    }

    #[test]
    #[serial]
    fn test_pool_settings_defaults() {
        env::remove_var("DATABASE_MAX_CONNECTIONS");
        env::remove_var("DATABASE_MIN_CONNECTIONS");
        env::remove_var("DATABASE_ACQUIRE_TIMEOUT_SECS");
        env::remove_var("DATABASE_IDLE_TIMEOUT_SECS");
        env::remove_var("DATABASE_MAX_LIFETIME_SECS");

        let settings = PoolSettings::from_env().unwrap();

        assert_eq!(settings.max_connections, DEFAULT_MAX_CONNECTIONS);
        assert_eq!(settings.min_connections, DEFAULT_MIN_CONNECTIONS);
        assert_eq!(settings.acquire_timeout_secs, DEFAULT_ACQUIRE_TIMEOUT_SECS);
        assert_eq!(settings.idle_timeout_secs, Some(DEFAULT_IDLE_TIMEOUT_SECS));
        assert_eq!(settings.max_lifetime_secs, Some(DEFAULT_MAX_LIFETIME_SECS));
    }

    #[test]
    #[serial]
    fn test_pool_settings_custom_values() {
        env::set_var("DATABASE_MAX_CONNECTIONS", "15");
        env::set_var("DATABASE_MIN_CONNECTIONS", "5");
        env::set_var("DATABASE_ACQUIRE_TIMEOUT_SECS", "8");
        env::set_var("DATABASE_IDLE_TIMEOUT_SECS", "900");
        env::set_var("DATABASE_MAX_LIFETIME_SECS", "3600");

        let settings = PoolSettings::from_env().unwrap();

        assert_eq!(settings.max_connections, 15);
        assert_eq!(settings.min_connections, 5);
        assert_eq!(settings.acquire_timeout_secs, 8);
        assert_eq!(settings.idle_timeout_secs, Some(900));
        assert_eq!(settings.max_lifetime_secs, Some(3_600));

        env::remove_var("DATABASE_MAX_CONNECTIONS");
        env::remove_var("DATABASE_MIN_CONNECTIONS");
        env::remove_var("DATABASE_ACQUIRE_TIMEOUT_SECS");
        env::remove_var("DATABASE_IDLE_TIMEOUT_SECS");
        env::remove_var("DATABASE_MAX_LIFETIME_SECS");
    }

    #[test]
    #[serial]
    fn test_pool_settings_invalid_relationship() {
        env::set_var("DATABASE_MAX_CONNECTIONS", "5");
        env::set_var("DATABASE_MIN_CONNECTIONS", "10");

        let err = PoolSettings::from_env().unwrap_err();
        assert!(err.to_string().contains("cannot exceed"));

        env::remove_var("DATABASE_MAX_CONNECTIONS");
        env::remove_var("DATABASE_MIN_CONNECTIONS");
    }

    #[test]
    #[serial]
    fn test_pool_settings_zero_disables_timeouts() {
        env::set_var("DATABASE_IDLE_TIMEOUT_SECS", "0");
        env::set_var("DATABASE_MAX_LIFETIME_SECS", "0");

        let settings = PoolSettings::from_env().unwrap();

        assert_eq!(settings.idle_timeout_secs, None);
        assert_eq!(settings.max_lifetime_secs, None);

        env::remove_var("DATABASE_IDLE_TIMEOUT_SECS");
        env::remove_var("DATABASE_MAX_LIFETIME_SECS");
    }
}
