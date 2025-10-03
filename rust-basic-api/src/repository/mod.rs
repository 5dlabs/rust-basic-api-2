//! Database connectivity utilities.

use sqlx::postgres::{PgPool, PgPoolOptions};

/// Type alias for the application's database connection pool.
pub type DbPool = PgPool;

/// Create a lazily-initialised `PostgreSQL` connection pool.
///
/// # Errors
///
/// Returns a [`sqlx::Error`] when the pool cannot be constructed.
pub fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(database_url)
}
