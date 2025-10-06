#![allow(dead_code)]

use std::sync::Once;

use sqlx::{PgPool, Postgres, Transaction};

static INIT: Once = Once::new();

/// Initialise the test database by creating a pool and running all migrations.
///
/// # Panics
///
/// Panics if `TEST_DATABASE_URL` or `DATABASE_URL` are not set or if the
/// connection pool and migrations cannot be initialised.
pub async fn setup_test_database() -> PgPool {
    INIT.call_once(|| {
        dotenv::from_filename(".env.test").ok();
        dotenv::dotenv().ok();
    });

    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("DATABASE_URL must be set for tests");

    let pool = super::create_pool(&database_url)
        .await
        .expect("Failed to create test database pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations for tests");

    pool
}

/// Start a transaction that can be rolled back after the test completes.
///
/// # Panics
///
/// Panics if starting the database transaction fails.
pub async fn transaction(pool: &PgPool) -> Transaction<'_, Postgres> {
    pool.begin()
        .await
        .expect("Failed to begin database transaction")
}

/// Remove data inserted during tests while resetting identity counters.
///
/// # Panics
///
/// Panics if the cleanup query fails to execute against the database.
pub async fn cleanup_database(pool: &PgPool) {
    sqlx::query("TRUNCATE TABLE users RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("Failed to cleanup database state");
}
