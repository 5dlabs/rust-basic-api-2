//! Database test helpers for integration testing scenarios.

use std::{env, sync::Once};

use crate::{
    config::Config,
    repository::{create_pool, PoolSettings, MIGRATOR},
};

use sqlx::{Connection, PgConnection, PgPool, Postgres, Transaction};
use url::Url;

static INIT: Once = Once::new();

fn load_test_environment() {
    INIT.call_once(|| {
        dotenv::from_filename(".env.test").ok();

        if env::var("DATABASE_URL").is_err() {
            if let Ok(test_url) = env::var("TEST_DATABASE_URL") {
                env::set_var("DATABASE_URL", test_url);
            } else if let Some(composed_url) = build_database_url_from_components() {
                env::set_var("DATABASE_URL", composed_url);
            }
        }

        if env::var("TEST_DATABASE_URL").is_err() {
            if let Some(composed_url) = build_database_url_from_components() {
                env::set_var("TEST_DATABASE_URL", composed_url);
            }
        }
    });
}

fn build_database_url_from_components() -> Option<String> {
    let scheme = env::var("TEST_DB_SCHEME").ok()?;
    let user = env::var("TEST_DB_USER").ok()?;
    let password = env::var("TEST_DB_PASSWORD").ok()?;
    let host = env::var("TEST_DB_HOST").ok()?;
    let port = env::var("TEST_DB_PORT").ok()?;
    let name = env::var("TEST_DB_NAME").ok()?;

    Some(format!("{scheme}://{user}:{password}@{host}:{port}/{name}"))
}

async fn ensure_database_exists(database_url: &str) {
    let mut url = Url::parse(database_url).expect("DATABASE_URL must be a valid URL");
    let db_name = url.path().trim_start_matches('/').to_string();

    assert!(
        !db_name.is_empty(),
        "DATABASE_URL must contain a database name"
    );

    url.set_path("/postgres");
    let admin_url = url.to_string();

    let mut connection = PgConnection::connect(&admin_url)
        .await
        .expect("Failed to connect to admin database");

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)",
    )
    .bind(&db_name)
    .fetch_one(&mut connection)
    .await
    .expect("Failed to determine if database exists");

    if !exists {
        let escaped_name = db_name.replace('"', "\"\"");
        let create_statement = format!("CREATE DATABASE \"{escaped_name}\"");
        sqlx::query(&create_statement)
            .execute(&mut connection)
            .await
            .expect("Failed to create test database");
    }

    connection
        .close()
        .await
        .expect("Failed to close admin connection");
}

/// Create and migrate a database connection pool for testing purposes.
///
/// # Panics
///
/// Panics if required configuration environment variables are missing or if the
/// database cannot be created, migrated, or the pool fails to initialize.
pub async fn setup_test_database() -> PgPool {
    load_test_environment();

    let config = Config::from_env().expect("Configuration should load for tests");
    let database_url = &config.database.url;

    ensure_database_exists(database_url).await;

    let pool_settings = PoolSettings::from(&config.database);
    let pool = create_pool(database_url, &pool_settings)
        .await
        .expect("Failed to establish test database pool");

    MIGRATOR
        .run(&pool)
        .await
        .expect("Failed to run database migrations for tests");

    pool
}

/// Create a transaction for isolating database state within a test case.
///
/// # Panics
///
/// Panics if beginning the transaction fails.
pub async fn transaction(pool: &PgPool) -> Transaction<'_, Postgres> {
    pool.begin()
        .await
        .expect("Failed to start database transaction")
}

/// Remove all records from tables that participate in tests.
///
/// # Panics
///
/// Panics if truncating tables fails.
pub async fn cleanup_database(pool: &PgPool) {
    sqlx::query("TRUNCATE TABLE users RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("Failed to truncate users table");
}
