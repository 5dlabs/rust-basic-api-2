use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Once;

static INIT: Once = Once::new();

/// Set up test database connection pool with migrations
///
/// # Panics
///
/// Panics if `DATABASE_URL` is not set in .env.test or if database connection fails
pub async fn setup_test_database() -> PgPool {
    INIT.call_once(|| {
        dotenv::from_filename(".env.test").ok();
    });

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env.test");

    let pool = super::create_pool(&database_url, 10).expect("Failed to create test database pool");

    // Run migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Begin a new database transaction for testing
///
/// # Panics
///
/// Panics if transaction cannot be started
pub async fn transaction(pool: &PgPool) -> Transaction<'_, Postgres> {
    pool.begin().await.expect("Failed to begin transaction")
}

/// Clean up database by truncating all tables
///
/// # Panics
///
/// Panics if cleanup fails
pub async fn cleanup_database(pool: &PgPool) {
    sqlx::query("TRUNCATE TABLE users RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("Failed to cleanup database");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires database connection"]
    async fn test_setup_test_database() {
        let _pool = setup_test_database().await;
        // Pool created successfully
    }

    #[tokio::test]
    #[ignore = "Requires database connection"]
    async fn test_transaction_creation() {
        let pool = setup_test_database().await;
        let tx = transaction(&pool).await;
        tx.rollback().await.unwrap();
    }

    #[tokio::test]
    #[ignore = "Requires database connection"]
    async fn test_cleanup_database() {
        let pool = setup_test_database().await;
        cleanup_database(&pool).await;
    }
}
