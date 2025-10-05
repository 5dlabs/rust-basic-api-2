use std::{str::FromStr, time::Duration};

use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

/// Create a `PostgreSQL` connection pool configured for production usage.
///
/// # Errors
///
/// Returns a [`sqlx::Error`] if the connection string cannot be parsed or the
/// connection attempt fails within the configured timeout.
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let options = PgConnectOptions::from_str(database_url)?.application_name("rust-basic-api");

    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .test_before_acquire(true)
        .connect_with(options)
        .await
}

#[cfg(test)]
pub mod test_utils;
