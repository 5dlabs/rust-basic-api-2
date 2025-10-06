#![allow(dead_code)]

use std::{env, sync::Once};

use sqlx::{PgPool, Postgres, Transaction};

static INIT: Once = Once::new();
const POSTGRES_SCHEME: &str = "postgresql://";

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

    let database_url = resolve_database_url();

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

fn resolve_database_url() -> String {
    if let Some(url) = env_url("TEST_DATABASE_URL") {
        return url;
    }

    if let Some(url) = env_url("DATABASE_URL") {
        return url;
    }

    let user = env_value_or_default("TEST_DB_USER", "postgres");
    let password = env_value_or_default("TEST_DB_PASSWORD", "postgres");
    let host = env_value_or_default("TEST_DB_HOST", "localhost");
    let port = env_value_or_default("TEST_DB_PORT", "5432");
    let name = env_value_or_default("TEST_DB_NAME", "rust_basic_api_test");

    format!("{POSTGRES_SCHEME}{user}:{password}@{host}:{port}/{name}")
}

fn env_url(key: &str) -> Option<String> {
    env::var(key).ok().and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed.contains("${") {
            None
        } else {
            let mut parts = trimmed.splitn(2, '@');
            let before_at = parts.next()?;
            parts.next()?;

            let user_section = before_at.trim_start_matches(POSTGRES_SCHEME);
            if user_section.is_empty() {
                return None;
            }

            let user = user_section.split(':').next().unwrap_or("");
            if user.is_empty() || user.eq_ignore_ascii_case("root") {
                return None;
            }

            Some(trimmed.to_string())
        }
    })
}

fn env_value_or_default(key: &str, default: &str) -> String {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && !value.contains("${"))
        .unwrap_or_else(|| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_resolve_database_url_defaults_with_placeholders() {
        env::set_var("TEST_DATABASE_URL", "${DATABASE_URL}");
        env::set_var(
            "DATABASE_URL",
            "postgresql://${TEST_DB_USER:-postgres}:${TEST_DB_PASSWORD:-postgres}@localhost:5432/rust_basic_api_test",
        );
        env::remove_var("TEST_DB_USER");
        env::remove_var("TEST_DB_PASSWORD");
        env::remove_var("TEST_DB_HOST");
        env::remove_var("TEST_DB_PORT");
        env::remove_var("TEST_DB_NAME");

        let url = resolve_database_url();

        let default_user = "postgres";
        let default_password = "postgres";
        let default_host = "localhost";
        let default_port = "5432";
        let expected = format!(
            "{POSTGRES_SCHEME}{default_user}:{default_password}@{default_host}:{default_port}/rust_basic_api_test"
        );

        assert_eq!(url, expected);

        env::remove_var("TEST_DATABASE_URL");
        env::remove_var("DATABASE_URL");
    }
}
