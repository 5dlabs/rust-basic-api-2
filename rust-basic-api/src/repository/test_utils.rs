use std::{
    fs::File,
    io::{BufRead, BufReader},
    sync::Once,
};

use crate::config::database_url_from_env;
use anyhow::{Context, Result};
use sqlx::{migrate::Migrator, PgPool, Postgres, Transaction};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
static INIT: Once = Once::new();

/// Load test environment configuration and return a pooled Postgres connection.
///
/// # Errors
///
/// Returns an error if test environment variables are missing, the database
/// pool cannot be created, or any of the schema reset queries fail.
pub async fn setup_test_database() -> Result<PgPool> {
    ensure_test_env();

    let database_url = database_url_from_env()
        .context("configure TEST_DATABASE_URL, DATABASE_URL, or component variables for test database operations")?;

    let pool = super::create_pool(&database_url)
        .await
        .context("failed to create test database pool")?;

    // Reset migrations and target tables to guarantee a clean schema for every test run.
    sqlx::query("DROP TRIGGER IF EXISTS update_users_updated_at ON users")
        .execute(&pool)
        .await
        .context("failed to drop existing trigger before migrations")?;
    sqlx::query("DROP FUNCTION IF EXISTS update_updated_at_column")
        .execute(&pool)
        .await
        .context("failed to drop existing trigger function before migrations")?;
    sqlx::query("DROP TABLE IF EXISTS users CASCADE")
        .execute(&pool)
        .await
        .context("failed to drop users table before migrations")?;
    sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations CASCADE")
        .execute(&pool)
        .await
        .context("failed to reset migrations history before migrations")?;

    MIGRATOR
        .run(&pool)
        .await
        .context("failed to run database migrations for tests")?;

    Ok(pool)
}

/// Ensure the `.env.test` file has been loaded without overriding existing
/// environment variables. Safe to call multiple times.
pub fn ensure_test_env() {
    INIT.call_once(|| {
        load_env_if_present(".env.test");
    });
}

fn load_env_if_present(path: &str) {
    if let Ok(file) = File::open(path) {
        for line in BufReader::new(file).lines().map_while(Result::ok) {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let Some((key, value)) = trimmed.split_once('=') else {
                continue;
            };

            if std::env::var_os(key).is_none() {
                std::env::set_var(key, value);
            }
        }
    }
}

/// Begin a transaction for isolating database side effects inside a test.
///
/// # Errors
///
/// Returns an error if acquiring a database connection or starting the
/// transaction fails.
#[allow(dead_code)]
pub async fn transaction(pool: &PgPool) -> Result<Transaction<'_, Postgres>> {
    pool.begin()
        .await
        .context("failed to begin database transaction for test")
}

/// Cleanup database tables touched during tests to maintain isolation.
///
/// # Errors
///
/// Returns an error if truncating or resetting the target tables fails.
pub async fn cleanup_database(pool: &PgPool) -> Result<()> {
    sqlx::query(
        "DO $$
        BEGIN
            IF EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = 'public' AND table_name = 'users'
            ) THEN
                EXECUTE 'TRUNCATE TABLE users RESTART IDENTITY CASCADE';
            END IF;
        END $$;",
    )
    .execute(pool)
    .await
    .context("failed to truncate users table during test cleanup")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use sqlx::Row;
    use std::ffi::OsString;
    use tempfile::NamedTempFile;

    struct EnvSnapshot {
        saved: Vec<(String, Option<OsString>)>,
    }

    impl EnvSnapshot {
        fn new(keys: &[&str]) -> Self {
            let saved = keys
                .iter()
                .map(|&key| (key.to_owned(), std::env::var_os(key)))
                .collect();

            Self { saved }
        }
    }

    impl Drop for EnvSnapshot {
        fn drop(&mut self) {
            for (key, value) in self.saved.drain(..) {
                if let Some(existing) = value {
                    std::env::set_var(key, existing);
                } else {
                    std::env::remove_var(key);
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_load_env_if_present_sets_missing_values() {
        let temp = NamedTempFile::new().expect("temporary file should be created");
        std::fs::write(
            temp.path(),
            "TEST_UTILS_ALPHA=value_alpha\n# comment\nINVALID\nTEST_UTILS_BRAVO=value_bravo\n",
        )
        .expect("write to temporary env file should succeed");

        let _snapshot = EnvSnapshot::new(&["TEST_UTILS_ALPHA", "TEST_UTILS_BRAVO"]);
        std::env::remove_var("TEST_UTILS_ALPHA");
        std::env::remove_var("TEST_UTILS_BRAVO");

        load_env_if_present(temp.path().to_str().expect("path is valid UTF-8"));

        assert_eq!(std::env::var("TEST_UTILS_ALPHA").unwrap(), "value_alpha");
        assert_eq!(std::env::var("TEST_UTILS_BRAVO").unwrap(), "value_bravo");
    }

    #[test]
    #[serial]
    fn test_load_env_if_present_does_not_override_existing_values() {
        let temp = NamedTempFile::new().expect("temporary file should be created");
        std::fs::write(temp.path(), "TEST_UTILS_CHARLIE=fallback\n").expect("write should succeed");

        let _snapshot = EnvSnapshot::new(&["TEST_UTILS_CHARLIE"]);
        std::env::set_var("TEST_UTILS_CHARLIE", "original");

        load_env_if_present(temp.path().to_str().expect("path is valid UTF-8"));

        assert_eq!(std::env::var("TEST_UTILS_CHARLIE").unwrap(), "original");
    }

    #[tokio::test]
    #[serial]
    async fn test_setup_test_database_runs_migrations() {
        let pool = setup_test_database()
            .await
            .expect("setup_test_database should run migrations");

        let row = sqlx::query(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'users') AS present",
        )
        .fetch_one(&pool)
        .await
        .expect("information_schema query should succeed");

        assert!(row.get::<bool, _>("present"), "users table should exist");

        cleanup_database(&pool)
            .await
            .expect("cleanup_database should truncate tables");

        pool.close().await;
    }

    #[tokio::test]
    #[serial]
    async fn test_transaction_helper_rolls_back_changes() {
        let pool = setup_test_database()
            .await
            .expect("setup_test_database should run migrations");

        {
            let mut tx = transaction(&pool)
                .await
                .expect("transaction helper should start transaction");

            sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
                .bind("Tx User")
                .bind("tx-user@example.com")
                .execute(&mut tx)
                .await
                .expect("insert within transaction should succeed");
            // Transaction dropped without commit to trigger rollback.
        }

        let count: i64 = sqlx::query("SELECT COUNT(*) AS count FROM users")
            .fetch_one(&pool)
            .await
            .expect("count query should succeed")
            .get("count");

        assert_eq!(count, 0, "transaction drop should roll back changes");

        cleanup_database(&pool)
            .await
            .expect("cleanup_database should truncate tables");

        pool.close().await;
    }
}
