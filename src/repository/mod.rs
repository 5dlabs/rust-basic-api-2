//! Database repositories and data access utilities.

use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};

/// Create a configured `PostgreSQL` connection pool.
///
/// # Errors
///
/// Returns [`sqlx::Error`] when the pool cannot be created, such as when the
/// database URL is invalid or the database is unreachable.
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
}

#[cfg(test)]
pub mod test_utils;
