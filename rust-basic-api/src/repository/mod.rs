use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, time::Duration};
use tracing::warn;

/// Create a `PostgreSQL` connection pool with production-grade defaults.
///
/// The pool eagerly establishes a minimum number of connections and validates
/// connections before handing them to callers to ensure reliability under load.
///
/// # Errors
///
/// Returns an error if establishing the connection pool fails, such as when the
/// database URL is invalid or the database server is unreachable.
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let settings = PoolSettings::from_env();

    let options = PgPoolOptions::new()
        .max_connections(settings.max_connections)
        .min_connections(settings.min_connections)
        .acquire_timeout(Duration::from_secs(settings.acquire_timeout_secs))
        .test_before_acquire(settings.test_before_acquire);

    let options = if let Some(idle_timeout) = settings.idle_timeout_secs {
        options.idle_timeout(Duration::from_secs(idle_timeout))
    } else {
        options
    };

    let options = if let Some(max_lifetime) = settings.max_lifetime_secs {
        options.max_lifetime(Duration::from_secs(max_lifetime))
    } else {
        options
    };

    options.connect(database_url).await
}

#[cfg(test)]
pub mod test_utils;

#[derive(Debug, Clone, Copy)]
struct PoolSettings {
    max_connections: u32,
    min_connections: u32,
    acquire_timeout_secs: u64,
    idle_timeout_secs: Option<u64>,
    max_lifetime_secs: Option<u64>,
    test_before_acquire: bool,
}

impl Default for PoolSettings {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            acquire_timeout_secs: 3,
            idle_timeout_secs: Some(600),
            max_lifetime_secs: Some(1800),
            test_before_acquire: true,
        }
    }
}

impl PoolSettings {
    fn from_env() -> Self {
        let mut settings = Self::default();

        settings.max_connections =
            Self::parse_u32("DATABASE_POOL_MAX_CONNECTIONS", settings.max_connections, 1);
        settings.min_connections =
            Self::parse_u32("DATABASE_POOL_MIN_CONNECTIONS", settings.min_connections, 0);

        if settings.min_connections > settings.max_connections {
            warn!(
                min = settings.min_connections,
                max = settings.max_connections,
                "DATABASE_POOL_MIN_CONNECTIONS greater than max; clamping to max"
            );
            settings.min_connections = settings.max_connections;
        }

        settings.acquire_timeout_secs = Self::parse_u64(
            "DATABASE_POOL_ACQUIRE_TIMEOUT_SECS",
            settings.acquire_timeout_secs,
            1,
        );

        settings.idle_timeout_secs = Self::parse_optional_u64(
            "DATABASE_POOL_IDLE_TIMEOUT_SECS",
            settings.idle_timeout_secs,
        );

        settings.max_lifetime_secs = Self::parse_optional_u64(
            "DATABASE_POOL_MAX_LIFETIME_SECS",
            settings.max_lifetime_secs,
        );

        settings.test_before_acquire = Self::parse_bool(
            "DATABASE_POOL_TEST_BEFORE_ACQUIRE",
            settings.test_before_acquire,
        );

        settings
    }

    fn parse_u32(key: &str, default: u32, minimum: u32) -> u32 {
        match env::var(key) {
            Ok(value) => match value.trim().parse::<u32>() {
                Ok(parsed) if parsed >= minimum => parsed,
                Ok(_) => {
                    warn!(
                        key = %key,
                        value = %value,
                        minimum = minimum,
                        "value below minimum, using minimum"
                    );
                    minimum.max(default)
                }
                Err(error) => {
                    warn!(
                        key = %key,
                        value = %value,
                        error = %error,
                        "invalid value, using default"
                    );
                    default
                }
            },
            Err(_) => default,
        }
    }

    fn parse_u64(key: &str, default: u64, minimum: u64) -> u64 {
        match env::var(key) {
            Ok(value) => match value.trim().parse::<u64>() {
                Ok(parsed) if parsed >= minimum => parsed,
                Ok(_) => {
                    warn!(
                        key = %key,
                        value = %value,
                        minimum = minimum,
                        "value below minimum, using minimum"
                    );
                    minimum.max(default)
                }
                Err(error) => {
                    warn!(
                        key = %key,
                        value = %value,
                        error = %error,
                        "invalid value, using default"
                    );
                    default
                }
            },
            Err(_) => default,
        }
    }

    fn parse_optional_u64(key: &str, default: Option<u64>) -> Option<u64> {
        match env::var(key) {
            Ok(value) => {
                let trimmed = value.trim();
                if trimmed.eq_ignore_ascii_case("none") || trimmed == "0" {
                    return None;
                }

                match trimmed.parse::<u64>() {
                    Ok(parsed) if parsed > 0 => Some(parsed),
                    Ok(_) => {
                        warn!(key = %key, value = %value, "value must be positive; disabling setting");
                        None
                    }
                    Err(error) => {
                        warn!(
                            key = %key,
                            value = %value,
                            error = %error,
                            "invalid value, using default"
                        );
                        default
                    }
                }
            }
            Err(_) => default,
        }
    }

    fn parse_bool(key: &str, default: bool) -> bool {
        match env::var(key) {
            Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => true,
                "false" | "0" | "no" | "off" => false,
                other => {
                    warn!(key = %key, value = %other, "invalid boolean value, using default");
                    default
                }
            },
            Err(_) => default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::database_url_from_env, repository::test_utils};
    use serial_test::serial;
    use std::{env, ffi::OsString};

    struct EnvGuard {
        saved: Vec<(String, Option<OsString>)>,
    }

    impl EnvGuard {
        fn new(keys: &[&str]) -> Self {
            let saved = keys
                .iter()
                .map(|&key| (key.to_owned(), env::var_os(key)))
                .collect();

            Self { saved }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in self.saved.drain(..) {
                if let Some(existing) = value {
                    env::set_var(key, existing);
                } else {
                    env::remove_var(key);
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_pool_settings_defaults() {
        let keys = [
            "DATABASE_POOL_MAX_CONNECTIONS",
            "DATABASE_POOL_MIN_CONNECTIONS",
            "DATABASE_POOL_ACQUIRE_TIMEOUT_SECS",
            "DATABASE_POOL_IDLE_TIMEOUT_SECS",
            "DATABASE_POOL_MAX_LIFETIME_SECS",
            "DATABASE_POOL_TEST_BEFORE_ACQUIRE",
        ];
        let _guard = EnvGuard::new(&keys);
        for key in &keys {
            env::remove_var(key);
        }

        let settings = PoolSettings::from_env();

        assert_eq!(settings.max_connections, 10);
        assert_eq!(settings.min_connections, 2);
        assert_eq!(settings.acquire_timeout_secs, 3);
        assert_eq!(settings.idle_timeout_secs, Some(600));
        assert_eq!(settings.max_lifetime_secs, Some(1800));
        assert!(settings.test_before_acquire);
    }

    #[test]
    #[serial]
    fn test_pool_settings_env_overrides() {
        let keys = [
            "DATABASE_POOL_MAX_CONNECTIONS",
            "DATABASE_POOL_MIN_CONNECTIONS",
            "DATABASE_POOL_ACQUIRE_TIMEOUT_SECS",
            "DATABASE_POOL_IDLE_TIMEOUT_SECS",
            "DATABASE_POOL_MAX_LIFETIME_SECS",
            "DATABASE_POOL_TEST_BEFORE_ACQUIRE",
        ];
        let _guard = EnvGuard::new(&keys);

        env::set_var("DATABASE_POOL_MAX_CONNECTIONS", "20");
        env::set_var("DATABASE_POOL_MIN_CONNECTIONS", "5");
        env::set_var("DATABASE_POOL_ACQUIRE_TIMEOUT_SECS", "10");
        env::set_var("DATABASE_POOL_IDLE_TIMEOUT_SECS", "900");
        env::set_var("DATABASE_POOL_MAX_LIFETIME_SECS", "0");
        env::set_var("DATABASE_POOL_TEST_BEFORE_ACQUIRE", "false");

        let settings = PoolSettings::from_env();

        assert_eq!(settings.max_connections, 20);
        assert_eq!(settings.min_connections, 5);
        assert_eq!(settings.acquire_timeout_secs, 10);
        assert_eq!(settings.idle_timeout_secs, Some(900));
        assert_eq!(settings.max_lifetime_secs, None);
        assert!(!settings.test_before_acquire);
    }

    #[test]
    #[serial]
    fn test_pool_settings_invalid_values_fallback() {
        let keys = [
            "DATABASE_POOL_MAX_CONNECTIONS",
            "DATABASE_POOL_MIN_CONNECTIONS",
            "DATABASE_POOL_ACQUIRE_TIMEOUT_SECS",
            "DATABASE_POOL_IDLE_TIMEOUT_SECS",
            "DATABASE_POOL_TEST_BEFORE_ACQUIRE",
        ];
        let _guard = EnvGuard::new(&keys);

        env::set_var("DATABASE_POOL_MAX_CONNECTIONS", "0");
        env::set_var("DATABASE_POOL_MIN_CONNECTIONS", "100");
        env::set_var("DATABASE_POOL_ACQUIRE_TIMEOUT_SECS", "not-a-number");
        env::set_var("DATABASE_POOL_IDLE_TIMEOUT_SECS", "none");
        env::set_var("DATABASE_POOL_TEST_BEFORE_ACQUIRE", "invalid");

        let settings = PoolSettings::from_env();

        assert_eq!(settings.max_connections, 10, "fallback to default max");
        assert_eq!(
            settings.min_connections, 10,
            "min should clamp to effective max when configured above",
        );
        assert_eq!(
            settings.acquire_timeout_secs, 3,
            "invalid acquire timeout should fallback to default",
        );
        assert_eq!(
            settings.idle_timeout_secs, None,
            "string 'none' disables idle timeout"
        );
        assert!(
            settings.test_before_acquire,
            "invalid boolean falls back to default true"
        );
    }

    #[test]
    #[serial]
    fn test_pool_settings_parsing_edge_cases() {
        let keys = [
            "DATABASE_POOL_MAX_CONNECTIONS",
            "DATABASE_POOL_MIN_CONNECTIONS",
            "DATABASE_POOL_ACQUIRE_TIMEOUT_SECS",
            "DATABASE_POOL_IDLE_TIMEOUT_SECS",
            "DATABASE_POOL_MAX_LIFETIME_SECS",
            "DATABASE_POOL_TEST_BEFORE_ACQUIRE",
        ];
        let _guard = EnvGuard::new(&keys);

        env::set_var("DATABASE_POOL_MAX_CONNECTIONS", "invalid-u32");
        env::set_var("DATABASE_POOL_MIN_CONNECTIONS", "invalid-u32");
        env::set_var("DATABASE_POOL_ACQUIRE_TIMEOUT_SECS", "invalid-u64");
        env::set_var("DATABASE_POOL_IDLE_TIMEOUT_SECS", "00");
        env::set_var("DATABASE_POOL_MAX_LIFETIME_SECS", "invalid-u64");
        env::set_var("DATABASE_POOL_TEST_BEFORE_ACQUIRE", "yes");

        let settings = PoolSettings::from_env();

        assert_eq!(settings.max_connections, 10, "invalid max uses default");
        assert_eq!(settings.min_connections, 2, "invalid min uses default");
        assert_eq!(
            settings.acquire_timeout_secs, 3,
            "invalid acquire timeout uses default",
        );
        assert_eq!(
            settings.idle_timeout_secs, None,
            "string with leading zero disables idle timeout",
        );
        assert_eq!(
            settings.max_lifetime_secs,
            Some(1800),
            "invalid max lifetime falls back to default",
        );
        assert!(
            settings.test_before_acquire,
            "truthy strings should enable test_before_acquire",
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_create_pool_with_timeouts_disabled() {
        let keys = [
            "DATABASE_POOL_MAX_CONNECTIONS",
            "DATABASE_POOL_MIN_CONNECTIONS",
            "DATABASE_POOL_ACQUIRE_TIMEOUT_SECS",
            "DATABASE_POOL_IDLE_TIMEOUT_SECS",
            "DATABASE_POOL_MAX_LIFETIME_SECS",
            "DATABASE_POOL_TEST_BEFORE_ACQUIRE",
        ];
        let _guard = EnvGuard::new(&keys);

        env::set_var("DATABASE_POOL_MAX_CONNECTIONS", "4");
        env::set_var("DATABASE_POOL_MIN_CONNECTIONS", "1");
        env::set_var("DATABASE_POOL_ACQUIRE_TIMEOUT_SECS", "5");
        env::set_var("DATABASE_POOL_IDLE_TIMEOUT_SECS", "none");
        env::set_var("DATABASE_POOL_MAX_LIFETIME_SECS", "none");
        env::set_var("DATABASE_POOL_TEST_BEFORE_ACQUIRE", "on");

        test_utils::ensure_test_env();
        let database_url = database_url_from_env()
            .expect("test database url must be configured for repository tests");

        let pool = create_pool(&database_url)
            .await
            .expect("create_pool should succeed with disabled timeouts");

        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .expect("basic connectivity check should succeed");

        pool.close().await;
    }
}
