use std::str::FromStr;
use std::time::Duration;

use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

/// Create a lazily-initialised `PostgreSQL` connection pool.
pub fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let options = PgConnectOptions::from_str(database_url)?;

    Ok(PgPoolOptions::new()
        .max_connections(5)
        .min_connections(0)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .connect_lazy_with(options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_pool_accepts_valid_url() {
        let pool = create_pool("postgresql://localhost:5432/example_db")
            .expect("pool should be created lazily with valid url");

        assert!(!pool.is_closed());
    }

    #[tokio::test]
    async fn create_pool_rejects_invalid_url() {
        let error = create_pool("not-a-valid-postgres-url").expect_err("invalid url must fail");

        assert!(matches!(error, sqlx::Error::Configuration(_)));
    }
}
