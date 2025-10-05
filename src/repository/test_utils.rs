use std::sync::Once;

use sqlx::{PgPool, Postgres, Transaction};

use super::create_pool;

static INIT: Once = Once::new();

/// Initialise a connection pool for integration tests and run migrations.
///
/// # Panics
///
/// Panics if the database URL environment variables are not set or if the
/// connection pool or migrations fail to initialise.
pub async fn setup_test_database() -> PgPool {
    INIT.call_once(|| {
        let _ = dotenv::from_filename(".env.test");
    });

    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("DATABASE_URL or TEST_DATABASE_URL must be set in .env.test");

    let pool = create_pool(&database_url)
        .await
        .expect("failed to create test database pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run database migrations");

    pool
}

/// Convenience helper for creating a transaction per test for isolation.
///
/// # Panics
///
/// Panics if a new transaction cannot be started on the provided pool.
pub async fn transaction(pool: &PgPool) -> Transaction<'_, Postgres> {
    pool.begin()
        .await
        .expect("failed to begin database transaction")
}

/// Remove all data from tables to keep tests idempotent.
///
/// # Panics
///
/// Panics if the cleanup query execution fails.
pub async fn cleanup_database(pool: &PgPool) {
    sqlx::query("TRUNCATE TABLE users RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("failed to clean up database");
}
