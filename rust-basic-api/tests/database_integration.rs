//! Integration tests validating database schema and behaviour.

use chrono::{DateTime, Utc};
use rust_basic_api::repository::test_utils::{cleanup_database, setup_test_database, transaction};
use rust_basic_api::repository::MIGRATOR;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_users_table_exists() {
    let pool = setup_test_database().await;

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
              AND table_name = 'users'
        )",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to verify users table existence");

    assert!(exists, "users table should be present after migrations");

    cleanup_database(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_users_indexes_exist() {
    let pool = setup_test_database().await;

    let indexes = sqlx::query_scalar::<_, String>(
        "SELECT indexname
         FROM pg_indexes
         WHERE schemaname = 'public'
           AND tablename = 'users'
           AND indexname IN ('idx_users_email', 'idx_users_created_at')",
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch index definitions");

    assert_eq!(
        indexes.len(),
        2,
        "Expected both email and created_at indexes"
    );

    cleanup_database(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_users_unique_constraint() {
    let pool = setup_test_database().await;
    let mut tx = transaction(&pool).await;

    sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
        .bind("User One")
        .bind("duplicate@example.com")
        .execute(&mut tx)
        .await
        .expect("Initial insert should succeed");

    let duplicate = sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
        .bind("User Two")
        .bind("duplicate@example.com")
        .execute(&mut tx)
        .await;

    assert!(
        duplicate.is_err(),
        "Duplicate email should violate unique constraint"
    );

    // Transaction will roll back automatically when dropped.
    drop(tx);
    cleanup_database(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_updated_at_trigger() {
    let pool = setup_test_database().await;

    let mut tx = transaction(&pool).await;

    let record_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",
    )
    .bind("Trigger Test")
    .bind("trigger@example.com")
    .fetch_one(&mut tx)
    .await
    .expect("Failed to insert user for trigger test");

    let (created_at, initial_updated_at): (DateTime<Utc>, DateTime<Utc>) =
        sqlx::query_as("SELECT created_at, updated_at FROM users WHERE id = $1")
            .bind(record_id)
            .fetch_one(&mut tx)
            .await
            .expect("Failed to fetch inserted timestamps");

    assert_eq!(created_at, initial_updated_at);

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    sqlx::query("UPDATE users SET name = $1 WHERE id = $2")
        .bind("Trigger Updated")
        .bind(record_id)
        .execute(&mut tx)
        .await
        .expect("Failed to update user for trigger test");

    let (_, updated_at): (DateTime<Utc>, DateTime<Utc>) =
        sqlx::query_as("SELECT created_at, updated_at FROM users WHERE id = $1")
            .bind(record_id)
            .fetch_one(&mut tx)
            .await
            .expect("Failed to fetch updated timestamps");

    assert!(
        updated_at > initial_updated_at,
        "updated_at should advance after UPDATE"
    );

    drop(tx);
    cleanup_database(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_migrations_idempotent() {
    let pool = setup_test_database().await;

    MIGRATOR
        .run(&pool)
        .await
        .expect("Re-running migrations should succeed");

    cleanup_database(&pool).await;
}
