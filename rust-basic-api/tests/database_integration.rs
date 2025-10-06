use chrono::{DateTime, Utc};
use rust_basic_api::repository::test_utils;
use serial_test::serial;
use tokio::time::{sleep, Duration};

#[tokio::test]
#[serial]
async fn test_users_table_exists() {
    let pool = test_utils::setup_test_database().await;

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_name = 'users'
        )",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to query information_schema for users table");

    assert!(exists, "users table should exist after migrations");

    test_utils::cleanup_database(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_indexes_exist() {
    let pool = test_utils::setup_test_database().await;

    let indexes = sqlx::query_scalar::<_, String>(
        "SELECT indexname FROM pg_indexes
         WHERE tablename = 'users'
         AND indexname IN ('idx_users_email', 'idx_users_created_at')",
    )
    .fetch_all(&pool)
    .await
    .expect("failed to fetch index information");

    assert_eq!(indexes.len(), 2, "expected both indexes to be present");

    test_utils::cleanup_database(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_user_insertion_and_retrieval() {
    let pool = test_utils::setup_test_database().await;

    let id: i32 = sqlx::query_scalar(
        "INSERT INTO users (name, email)
         VALUES ($1, $2)
         RETURNING id",
    )
    .bind("Test User")
    .bind("test@example.com")
    .fetch_one(&pool)
    .await
    .expect("failed to insert test user");

    assert!(id > 0, "inserted user ID should be positive");

    let user = sqlx::query_as::<_, (String, String, DateTime<Utc>, DateTime<Utc>)>(
        "SELECT name, email, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .expect("failed to fetch inserted user");

    let (name, email, created_at, updated_at) = user;

    assert_eq!(name, "Test User");
    assert_eq!(email, "test@example.com");
    assert!(created_at <= updated_at);

    test_utils::cleanup_database(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_email_unique_constraint() {
    let pool = test_utils::setup_test_database().await;

    sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
        .bind("User One")
        .bind("duplicate@example.com")
        .execute(&pool)
        .await
        .expect("failed to insert initial user");

    let result = sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
        .bind("User Two")
        .bind("duplicate@example.com")
        .execute(&pool)
        .await;

    assert!(
        result.is_err(),
        "duplicate email should violate unique constraint"
    );

    test_utils::cleanup_database(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_transaction_helper_rolls_back() {
    let pool = test_utils::setup_test_database().await;

    {
        let mut transaction = test_utils::transaction(&pool).await;

        sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
            .bind("Transient User")
            .bind("transient@example.com")
            .execute(&mut *transaction)
            .await
            .expect("failed to execute insert within transaction");

        transaction
            .rollback()
            .await
            .expect("failed to rollback transaction");
    }

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("failed to count users after rollback");

    assert_eq!(
        remaining, 0,
        "transaction rollback should remove inserted rows"
    );

    test_utils::cleanup_database(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_updated_at_trigger() {
    let pool = test_utils::setup_test_database().await;

    let id: i32 = sqlx::query_scalar(
        "INSERT INTO users (name, email)
         VALUES ($1, $2)
         RETURNING id",
    )
    .bind("Trigger User")
    .bind("trigger@example.com")
    .fetch_one(&pool)
    .await
    .expect("failed to insert trigger test user");

    sleep(Duration::from_millis(100)).await;

    sqlx::query("UPDATE users SET name = $1 WHERE id = $2")
        .bind("Trigger User Updated")
        .bind(id)
        .execute(&pool)
        .await
        .expect("failed to update user");

    let timestamps = sqlx::query_as::<_, (DateTime<Utc>, DateTime<Utc>)>(
        "SELECT created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .expect("failed to fetch timestamps");

    let (created_at, updated_at) = timestamps;

    assert!(
        updated_at > created_at,
        "updated_at should advance after update"
    );

    test_utils::cleanup_database(&pool).await;
}
