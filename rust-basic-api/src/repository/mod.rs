use std::str::FromStr;
use std::time::Duration;

use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

pub const DEFAULT_MAX_CONNECTIONS: u32 = 5;
pub const DEFAULT_MIN_CONNECTIONS: u32 = 0;
pub const DEFAULT_ACQUIRE_TIMEOUT_SECS: u64 = 5;
pub const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 300;

/// Create a lazily-initialised `PostgreSQL` connection pool.
pub fn create_pool(
    database_url: &str,
    max_connections: u32,
    min_connections: u32,
    acquire_timeout_secs: u64,
    idle_timeout_secs: u64,
) -> Result<PgPool, sqlx::Error> {
    let options = PgConnectOptions::from_str(database_url)?;

    Ok(PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(Duration::from_secs(acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(idle_timeout_secs))
        .connect_lazy_with(options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_pool_accepts_valid_url() {
        let pool = create_pool(
            "postgresql://localhost:5432/example_db",
            DEFAULT_MAX_CONNECTIONS,
            DEFAULT_MIN_CONNECTIONS,
            DEFAULT_ACQUIRE_TIMEOUT_SECS,
            DEFAULT_IDLE_TIMEOUT_SECS,
        )
        .expect("pool should be created lazily with valid url");

        assert!(!pool.is_closed());
    }

    #[tokio::test]
    async fn create_pool_rejects_invalid_url() {
        let error = create_pool(
            "not-a-valid-postgres-url",
            DEFAULT_MAX_CONNECTIONS,
            DEFAULT_MIN_CONNECTIONS,
            DEFAULT_ACQUIRE_TIMEOUT_SECS,
            DEFAULT_IDLE_TIMEOUT_SECS,
        )
        .expect_err("invalid url must fail");

        assert!(matches!(error, sqlx::Error::Configuration(_)));
    }
}
