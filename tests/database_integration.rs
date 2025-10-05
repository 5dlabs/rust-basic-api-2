use std::{env, time::Duration};

use chrono::{DateTime, Utc};
use serial_test::serial;
use sqlx::{PgPool, Row};

fn ensure_test_environment() {
    if env::var("TEST_DATABASE_URL").is_err() && env::var("DATABASE_URL").is_err() {
        if let Err(error) = dotenv::from_filename(".env.test") {
            panic!("failed to load test environment configuration: {error}");
        }
    }
}

fn database_url() -> String {
    ensure_test_environment();
    env::var("TEST_DATABASE_URL")
        .or_else(|_| env::var("DATABASE_URL"))
        .expect("DATABASE_URL or TEST_DATABASE_URL must be configured")
}

async fn setup_pool() -> PgPool {
    let pool = rust_basic_api::repository::create_pool(&database_url())
        .await
        .expect("failed to create PostgreSQL pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    pool
}

async fn cleanup(pool: &PgPool) {
    sqlx::query("TRUNCATE TABLE users RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("failed to truncate tables");
}

#[tokio::test]
#[serial]
async fn users_table_exists() {
    let pool = setup_pool().await;

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'users')",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to query information_schema");

    assert!(exists);

    cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn required_indexes_exist() {
    let pool = setup_pool().await;

    let indexes = sqlx::query(
        "SELECT indexname FROM pg_indexes WHERE tablename = 'users' AND indexname IN ($1, $2)",
    )
    .bind("idx_users_email")
    .bind("idx_users_created_at")
    .fetch_all(&pool)
    .await
    .expect("failed to fetch index metadata");

    assert_eq!(
        indexes.len(),
        2,
        "expected both users indexes to be present"
    );

    cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn user_insertion_returns_primary_key() {
    let pool = setup_pool().await;

    let id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",
    )
    .bind("Test User")
    .bind("test-user@example.com")
    .fetch_one(&pool)
    .await
    .expect("failed to insert user");

    assert!(id > 0);

    cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn email_unique_constraint_enforced() {
    let pool = setup_pool().await;

    sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
        .bind("User One")
        .bind("duplicate@example.com")
        .execute(&pool)
        .await
        .expect("first insert should succeed");

    let duplicate_result = sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
        .bind("User Two")
        .bind("duplicate@example.com")
        .execute(&pool)
        .await;

    assert!(
        duplicate_result.is_err(),
        "unique constraint should reject duplicates"
    );

    cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn updated_at_trigger_updates_timestamp() {
    let pool = setup_pool().await;

    let id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",
    )
    .bind("Trigger Test")
    .bind("trigger-test@example.com")
    .fetch_one(&pool)
    .await
    .expect("failed to insert user");

    tokio::time::sleep(Duration::from_millis(150)).await;

    sqlx::query("UPDATE users SET name = $1 WHERE id = $2")
        .bind("Updated Name")
        .bind(id)
        .execute(&pool)
        .await
        .expect("failed to update user");

    let row = sqlx::query("SELECT created_at, updated_at FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("failed to fetch timestamps");

    let created_at: DateTime<Utc> = row.get("created_at");
    let updated_at: DateTime<Utc> = row.get("updated_at");

    assert!(updated_at > created_at);

    cleanup(&pool).await;
}
