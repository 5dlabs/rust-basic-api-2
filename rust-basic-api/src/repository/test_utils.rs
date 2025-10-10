//! Test utilities for database testing.
//!
//! Provides helper functions for setting up test databases, managing transactions,
//! and cleaning up test data.

use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Once;

static INIT: Once = Once::new();

/// Sets up a test database pool and runs migrations.
///
/// This function:
/// - Loads configuration from `.env.test`
/// - Creates a connection pool
/// - Runs all pending migrations
///
/// # Panics
///
/// Panics if:
/// - `DATABASE_URL` is not set in `.env.test`
/// - Connection pool creation fails
/// - Migrations fail to run
#[must_use]
pub async fn setup_test_database() -> PgPool {
    INIT.call_once(|| {
        dotenv::from_filename(".env.test").ok();
    });

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env.test for integration tests");

    let pool = super::create_pool(&database_url, 5).expect("Failed to create test database pool");

    // Run migrations to ensure schema is up to date
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations in test setup");

    pool
}

/// Begins a new database transaction for test isolation.
///
/// Use this for tests that need to be isolated from each other.
/// The transaction should be rolled back after the test completes.
///
/// # Panics
///
/// Panics if the transaction cannot be started.
pub async fn transaction(pool: &PgPool) -> Transaction<'_, Postgres> {
    pool.begin()
        .await
        .expect("Failed to begin database transaction")
}

/// Cleans up all data from test tables.
///
/// Truncates the users table and resets sequences.
/// Use this between tests when you need a clean state.
///
/// # Panics
///
/// Panics if the cleanup query fails.
pub async fn cleanup_database(pool: &PgPool) {
    sqlx::query("TRUNCATE TABLE users RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("Failed to cleanup test database");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_creates_pool() {
        let pool = setup_test_database().await;
        // Pool size can be 0 for lazy pools, just verify it exists
        let _ = pool.size();
    }

    #[tokio::test]
    async fn test_transaction_can_be_created() {
        let pool = setup_test_database().await;
        let tx = transaction(&pool).await;
        // Transaction created successfully
        drop(tx); // Rollback
    }

    #[tokio::test]
    async fn test_cleanup_removes_data() {
        let pool = setup_test_database().await;

        // Insert test data
        sqlx::query("INSERT INTO users (name, email) VALUES ('Test', 'cleanup@test.com')")
            .execute(&pool)
            .await
            .expect("Failed to insert test data");

        // Cleanup
        cleanup_database(&pool).await;

        // Verify data is gone
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .expect("Failed to count users");

        assert_eq!(count, 0);
    }
}
