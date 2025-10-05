use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

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
    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .test_before_acquire(true)
        .connect(database_url)
        .await
}

#[cfg(test)]
pub mod test_utils;
