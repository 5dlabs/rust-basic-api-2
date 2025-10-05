//! Application configuration management.

use std::{env, net::IpAddr, time::Duration};

use dotenv::dotenv;

use anyhow::anyhow;

use crate::error::ConfigError;

/// Database-specific configuration settings.
#[derive(Debug, Clone)]
pub struct DatabaseSettings {
    /// Database connection string compatible with `PostgreSQL`.
    pub url: String,
    /// Maximum number of pooled connections that may be open simultaneously.
    pub max_connections: u32,
    /// Minimum number of pooled connections to keep warm.
    pub min_connections: u32,
    /// Maximum time to wait for a free connection from the pool.
    pub acquire_timeout: Duration,
    /// Maximum duration an idle connection is kept in the pool before being closed.
    pub idle_timeout: Option<Duration>,
    /// Maximum lifetime of individual connections before being recycled.
    pub max_lifetime: Option<Duration>,
    /// Timeout used when establishing brand-new `PostgreSQL` connections.
    pub connect_timeout: Duration,
}

/// Runtime configuration derived from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Database settings including connection string and pooling behaviour.
    pub database: DatabaseSettings,
    /// Host interface on which the server will listen.
    pub server_host: IpAddr,
    /// TCP port used by the HTTP server.
    pub server_port: u16,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Required environment variables:
    ///
    /// - `DATABASE_URL`
    ///
    /// Optional variables:
    ///
    /// - `SERVER_HOST` (defaults to `0.0.0.0`)
    /// - `SERVER_PORT` (defaults to `3000`)
    /// - `RUST_LOG` (handled by the tracing subscriber)
    ///
    /// # Errors
    ///
    /// Returns a [`ConfigError`] if:
    /// - `DATABASE_URL` environment variable is not set
    /// - `SERVER_HOST` contains an invalid IP address
    /// - `SERVER_PORT` contains an invalid port number
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").map_err(|_| ConfigError::Missing("DATABASE_URL"))?;

        let max_connections = parse_env_or_default("DB_MAX_CONNECTIONS", 10u32)?;
        let min_connections = parse_env_or_default("DB_MIN_CONNECTIONS", 2u32)?;
        if min_connections > max_connections {
            return Err(ConfigError::invalid(
                "DB_MIN_CONNECTIONS",
                anyhow!(
                    "DB_MIN_CONNECTIONS ({min_connections}) cannot exceed DB_MAX_CONNECTIONS ({max_connections})"
                ),
            ));
        }

        let acquire_timeout_seconds = parse_env_or_default("DB_ACQUIRE_TIMEOUT_SECONDS", 3u64)?;
        let idle_timeout_seconds = parse_env_or_default("DB_IDLE_TIMEOUT_SECONDS", 600u64)?;
        let max_lifetime_seconds = parse_env_or_default("DB_MAX_LIFETIME_SECONDS", 1800u64)?;
        let connect_timeout_seconds = parse_env_or_default("DB_CONNECT_TIMEOUT_SECONDS", 5u64)?;

        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string())
            .parse()
            .map_err(|err| ConfigError::invalid("SERVER_HOST", err))?;

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .map_err(|err| ConfigError::invalid("SERVER_PORT", err))?;

        let database = DatabaseSettings {
            url: database_url,
            max_connections,
            min_connections,
            acquire_timeout: Duration::from_secs(acquire_timeout_seconds),
            idle_timeout: optional_duration(idle_timeout_seconds),
            max_lifetime: optional_duration(max_lifetime_seconds),
            connect_timeout: Duration::from_secs(connect_timeout_seconds),
        };

        Ok(Self {
            database,
            server_host,
            server_port,
        })
    }
}

fn parse_env_or_default<T>(key: &'static str, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match env::var(key) {
        Ok(raw) => raw.parse().map_err(|err| ConfigError::invalid(key, err)),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::invalid(
            key,
            anyhow!("environment variable {key} is not valid unicode"),
        )),
    }
}

fn optional_duration(seconds: u64) -> Option<Duration> {
    if seconds == 0 {
        None
    } else {
        Some(Duration::from_secs(seconds))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::time::Duration;

    const DB_SCHEME: &str = "postgresql";
    const DB_SUFFIX: &str = "//user:password@localhost:5432/app_db";

    fn sample_database_url() -> String {
        format!("{DB_SCHEME}:{DB_SUFFIX}")
    }

    fn setup_env(db_url: &str, host: Option<&str>, port: Option<&str>) {
        env::set_var("DATABASE_URL", db_url);
        if let Some(h) = host {
            env::set_var("SERVER_HOST", h);
        } else {
            env::remove_var("SERVER_HOST");
        }
        if let Some(p) = port {
            env::set_var("SERVER_PORT", p);
        } else {
            env::remove_var("SERVER_PORT");
        }
    }

    fn cleanup_env() {
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_HOST");
        env::remove_var("SERVER_PORT");
        env::remove_var("DB_MAX_CONNECTIONS");
        env::remove_var("DB_MIN_CONNECTIONS");
        env::remove_var("DB_ACQUIRE_TIMEOUT_SECONDS");
        env::remove_var("DB_IDLE_TIMEOUT_SECONDS");
        env::remove_var("DB_MAX_LIFETIME_SECONDS");
        env::remove_var("DB_CONNECT_TIMEOUT_SECONDS");
    }

    #[test]
    #[serial]
    fn test_config_with_all_values() {
        let db_url = sample_database_url();
        setup_env(&db_url, Some("127.0.0.1"), Some("8080"));

        let config = Config::from_env().expect("Config should load");

        assert_eq!(config.database.url, db_url);
        assert_eq!(config.server_host.to_string(), "127.0.0.1");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.database.min_connections, 2);
        assert_eq!(config.database.acquire_timeout, Duration::from_secs(3));
        assert_eq!(config.database.idle_timeout, Some(Duration::from_secs(600)));
        assert_eq!(
            config.database.max_lifetime,
            Some(Duration::from_secs(1800))
        );
        assert_eq!(config.database.connect_timeout, Duration::from_secs(5));

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_with_defaults() {
        let db_url = sample_database_url();
        setup_env(&db_url, None, None);

        let config = Config::from_env().expect("Config should load");

        assert_eq!(config.database.url, db_url);
        assert_eq!(config.server_host.to_string(), "0.0.0.0");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.database.min_connections, 2);
        assert_eq!(config.database.acquire_timeout, Duration::from_secs(3));
        assert_eq!(config.database.idle_timeout, Some(Duration::from_secs(600)));
        assert_eq!(
            config.database.max_lifetime,
            Some(Duration::from_secs(1800))
        );
        assert_eq!(config.database.connect_timeout, Duration::from_secs(5));

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_missing_database_url() {
        cleanup_env();

        let result = Config::from_env();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::Missing("DATABASE_URL")));

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_invalid_host() {
        let db_url = sample_database_url();
        setup_env(&db_url, Some("invalid-host"), Some("3000"));

        let result = Config::from_env();

        assert!(result.is_err());
        if let Err(ConfigError::InvalidEnvVar { field, .. }) = result {
            assert_eq!(field, "SERVER_HOST");
        } else {
            panic!("Expected InvalidEnvVar error");
        }

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_invalid_port() {
        let db_url = sample_database_url();
        setup_env(&db_url, None, Some("invalid"));

        let result = Config::from_env();

        assert!(result.is_err());
        if let Err(ConfigError::InvalidEnvVar { field, .. }) = result {
            assert_eq!(field, "SERVER_PORT");
        } else {
            panic!("Expected InvalidEnvVar error");
        }

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_custom_pool_settings() {
        let db_url = sample_database_url();
        setup_env(&db_url, Some("127.0.0.1"), Some("3001"));
        env::set_var("DB_MAX_CONNECTIONS", "20");
        env::set_var("DB_MIN_CONNECTIONS", "4");
        env::set_var("DB_ACQUIRE_TIMEOUT_SECONDS", "10");
        env::set_var("DB_IDLE_TIMEOUT_SECONDS", "30");
        env::set_var("DB_MAX_LIFETIME_SECONDS", "60");
        env::set_var("DB_CONNECT_TIMEOUT_SECONDS", "7");

        let config = Config::from_env().expect("Config should load");

        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.database.min_connections, 4);
        assert_eq!(config.database.acquire_timeout, Duration::from_secs(10));
        assert_eq!(config.database.idle_timeout, Some(Duration::from_secs(30)));
        assert_eq!(config.database.max_lifetime, Some(Duration::from_secs(60)));
        assert_eq!(config.database.connect_timeout, Duration::from_secs(7));

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_invalid_pool_bounds() {
        let db_url = sample_database_url();
        setup_env(&db_url, None, None);
        env::set_var("DB_MAX_CONNECTIONS", "5");
        env::set_var("DB_MIN_CONNECTIONS", "10");

        let result = Config::from_env();

        assert!(
            matches!(result, Err(ConfigError::InvalidEnvVar { field, .. }) if field == "DB_MIN_CONNECTIONS")
        );

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_port_out_of_range() {
        let db_url = sample_database_url();
        setup_env(&db_url, None, Some("70000"));

        let result = Config::from_env();

        assert!(result.is_err());

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_clone() {
        let db_url = sample_database_url();
        setup_env(&db_url, Some("192.168.1.1"), Some("8888"));

        let config = Config::from_env().expect("Config should load");
        let cloned = config.clone();

        assert_eq!(config.database.url, cloned.database.url);
        assert_eq!(
            config.database.max_connections,
            cloned.database.max_connections
        );
        assert_eq!(config.server_host, cloned.server_host);
        assert_eq!(config.server_port, cloned.server_port);

        cleanup_env();
    }

    #[test]
    #[serial]
    fn test_config_debug() {
        let db_url = sample_database_url();
        setup_env(&db_url, None, None);

        let config = Config::from_env().expect("Config should load");
        let debug_str = format!("{config:?}");

        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("database"));
        assert!(debug_str.contains("server_host"));
        assert!(debug_str.contains("server_port"));

        cleanup_env();
    }
}
