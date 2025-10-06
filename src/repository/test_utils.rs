use std::sync::Once;

use sqlx::{PgPool, Postgres, Transaction};

static INIT: Once = Once::new();

fn default_database_url() -> String {
    let scheme = "postgresql";
    let user = "postgres";
    let password = "postgres";
    let host = "localhost";
    let port = 15432;
    let database = "rust_basic_api_test";

    format!("{scheme}://{user}:{password}@{host}:{port}/{database}")
}

/// Initialize the test database connection pool and apply migrations.
///
/// # Panics
///
/// Panics if the `DATABASE_URL` environment variable is missing or the
/// database pool and migrations cannot be completed successfully.
pub async fn setup_test_database() -> PgPool {
    INIT.call_once(|| {
        dotenv::from_filename(".env.test").ok();
    });

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        let fallback = default_database_url();
        std::env::set_var("DATABASE_URL", &fallback);
        fallback
    });

    let pool = super::create_pool(&database_url)
        .await
        .expect("Failed to create PostgreSQL pool for tests");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations for tests");

    pool
}

/// Create a transaction for isolating changes in tests.
///
/// # Panics
///
/// Panics if a transaction cannot be started on the provided pool.
#[allow(dead_code)]
pub async fn transaction(pool: &PgPool) -> Transaction<'_, Postgres> {
    pool.begin()
        .await
        .expect("Failed to begin transaction for test")
}

/// Remove all records from the users table and reset identifiers.
///
/// # Panics
///
/// Panics if the cleanup query cannot be executed successfully.
pub async fn cleanup_database(pool: &PgPool) {
    sqlx::query("TRUNCATE TABLE users RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("Failed to clean up database after test");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    async fn count_users(pool: &PgPool) -> i64 {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await
            .expect("failed to count users")
    }

    #[tokio::test]
    #[serial]
    async fn test_cleanup_database_clears_users() {
        let pool = setup_test_database().await;

        cleanup_database(&pool).await;

        sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
            .bind("Cleanup Test")
            .bind("cleanup@example.com")
            .execute(&pool)
            .await
            .expect("failed to insert user for cleanup test");

        assert_eq!(count_users(&pool).await, 1);

        cleanup_database(&pool).await;

        assert_eq!(count_users(&pool).await, 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_transaction_rolls_back_changes() {
        let pool = setup_test_database().await;
        cleanup_database(&pool).await;

        {
            let mut tx = transaction(&pool).await;

            sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
                .bind("Transaction Test")
                .bind("transaction@example.com")
                .execute(&mut *tx)
                .await
                .expect("failed to insert user inside transaction");

            let count_in_tx = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
                .fetch_one(&mut *tx)
                .await
                .expect("failed to count users inside transaction");

            assert_eq!(count_in_tx, 1);
        }

        assert_eq!(count_users(&pool).await, 0);

        cleanup_database(&pool).await;
    }
}
