use sqlx::{postgres::PgPoolOptions, PgPool};

/// Create a lazily-initialized `PostgreSQL` connection pool.
///
/// The pool will establish connections on demand, allowing the application
/// to start without requiring an immediate database connection while still
/// using real database interactions for live traffic.
pub fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_lazy(database_url)
}
