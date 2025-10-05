use std::{str::FromStr, time::Duration};

use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

/// Create a lazily-connected `PostgreSQL` connection pool.
///
/// # Errors
///
/// Returns a [`sqlx::Error`] if the database connection string cannot be parsed
/// into valid `PostgreSQL` connection options.
pub fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let options = PgConnectOptions::from_str(database_url)?.application_name("rust-basic-api");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy_with(options);

    Ok(pool)
}
