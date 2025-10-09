use sqlx::postgres::PgPoolOptions;

pub type DbPool = sqlx::PgPool;

/// Creates a lazy database connection pool.
///
/// # Errors
///
/// Returns `sqlx::Error` if the database URL is invalid or pool creation fails.
pub fn create_pool(database_url: &str, max_connections: u32) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect_lazy(database_url)
}
