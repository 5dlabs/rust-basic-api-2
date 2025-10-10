//! Database integration tests.
//!
//! These tests verify the database schema, migrations, and CRUD operations.
//!
//! **NOTE**: These tests require a running PostgreSQL instance.
//! Run with: `cargo test --test database_integration -- --ignored`

use rust_basic_api::repository::test_utils;
use serial_test::serial;
use sqlx::{types::chrono, Row};

/// Test that the users table exists with the correct schema.
#[tokio::test]
#[serial]
#[ignore = "requires running PostgreSQL instance"]
async fn test_users_table_exists() {
    let pool = test_utils::setup_test_database().await;

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public'
            AND table_name = 'users'
        )",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to check if users table exists");

    assert!(exists, "users table should exist");
}

/// Test that all required columns exist in the users table.
#[tokio::test]
#[serial]
#[ignore = "requires running PostgreSQL instance"]
async fn test_users_table_columns() {
    let pool = test_utils::setup_test_database().await;

    let columns: Vec<String> = sqlx::query_scalar(
        "SELECT column_name 
         FROM information_schema.columns 
         WHERE table_schema = 'public' 
         AND table_name = 'users'
         ORDER BY ordinal_position",
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch column names");

    assert_eq!(columns.len(), 5, "Should have exactly 5 columns");
    assert_eq!(columns[0], "id");
    assert_eq!(columns[1], "name");
    assert_eq!(columns[2], "email");
    assert_eq!(columns[3], "created_at");
    assert_eq!(columns[4], "updated_at");
}

/// Test that performance indexes exist.
#[tokio::test]
#[serial]
#[ignore = "requires running PostgreSQL instance"]
async fn test_indexes_exist() {
    let pool = test_utils::setup_test_database().await;

    let indexes: Vec<String> = sqlx::query_scalar(
        "SELECT indexname 
         FROM pg_indexes 
         WHERE tablename = 'users' 
         AND schemaname = 'public'
         AND indexname IN ('idx_users_email', 'idx_users_created_at')",
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch indexes");

    assert_eq!(
        indexes.len(),
        2,
        "Should have 2 performance indexes (email and created_at)"
    );
    assert!(
        indexes.contains(&"idx_users_email".to_string()),
        "Should have email index"
    );
    assert!(
        indexes.contains(&"idx_users_created_at".to_string()),
        "Should have created_at index"
    );
}

/// Test basic user insertion.
#[tokio::test]
#[serial]
#[ignore = "requires running PostgreSQL instance"]
async fn test_user_insertion() {
    let pool = test_utils::setup_test_database().await;
    test_utils::cleanup_database(&pool).await;

    let id: i32 = sqlx::query_scalar(
        "INSERT INTO users (name, email) 
         VALUES ($1, $2) 
         RETURNING id",
    )
    .bind("John Doe")
    .bind("john@example.com")
    .fetch_one(&pool)
    .await
    .expect("Failed to insert user");

    assert!(id > 0, "User ID should be greater than 0");
}

/// Test that email unique constraint is enforced.
#[tokio::test]
#[serial]
#[ignore = "requires running PostgreSQL instance"]
async fn test_email_unique_constraint() {
    let pool = test_utils::setup_test_database().await;
    test_utils::cleanup_database(&pool).await;

    // Insert first user
    sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
        .bind("User One")
        .bind("duplicate@example.com")
        .execute(&pool)
        .await
        .expect("First insert should succeed");

    // Try to insert second user with same email
    let result = sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
        .bind("User Two")
        .bind("duplicate@example.com")
        .execute(&pool)
        .await;

    assert!(
        result.is_err(),
        "Should fail to insert user with duplicate email"
    );

    // Verify error is a unique violation
    let error = result.unwrap_err();
    let db_error = error.as_database_error().expect("Should be database error");
    assert_eq!(
        db_error.code().as_deref(),
        Some("23505"),
        "Should be unique violation error"
    );
}

/// Test that timestamps are automatically set on insert.
#[tokio::test]
#[serial]
#[ignore = "requires running PostgreSQL instance"]
async fn test_timestamps_auto_set() {
    let pool = test_utils::setup_test_database().await;
    test_utils::cleanup_database(&pool).await;

    let row = sqlx::query(
        "INSERT INTO users (name, email) 
         VALUES ($1, $2) 
         RETURNING created_at, updated_at",
    )
    .bind("Timestamp Test")
    .bind("timestamp@example.com")
    .fetch_one(&pool)
    .await
    .expect("Failed to insert user");

    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

    assert!(
        created_at <= chrono::Utc::now(),
        "created_at should be in the past"
    );
    assert!(
        updated_at <= chrono::Utc::now(),
        "updated_at should be in the past"
    );
    assert_eq!(
        created_at, updated_at,
        "created_at and updated_at should be equal on insert"
    );
}

/// Test that `updated_at` trigger updates timestamp on update.
#[tokio::test]
#[serial]
#[ignore = "requires running PostgreSQL instance"]
async fn test_updated_at_trigger() {
    let pool = test_utils::setup_test_database().await;
    test_utils::cleanup_database(&pool).await;

    // Insert user
    let id: i32 = sqlx::query_scalar(
        "INSERT INTO users (name, email) 
         VALUES ($1, $2) 
         RETURNING id",
    )
    .bind("Trigger Test")
    .bind("trigger@example.com")
    .fetch_one(&pool)
    .await
    .expect("Failed to insert user");

    // Get initial timestamps
    let initial_row = sqlx::query("SELECT created_at, updated_at FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch initial timestamps");

    let created_at: chrono::DateTime<chrono::Utc> = initial_row.get("created_at");
    let initial_updated_at: chrono::DateTime<chrono::Utc> = initial_row.get("updated_at");

    // Wait a bit to ensure timestamp difference
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update user
    sqlx::query("UPDATE users SET name = $1 WHERE id = $2")
        .bind("Updated Name")
        .bind(id)
        .execute(&pool)
        .await
        .expect("Failed to update user");

    // Get new timestamps
    let updated_row = sqlx::query("SELECT created_at, updated_at FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch updated timestamps");

    let new_created_at: chrono::DateTime<chrono::Utc> = updated_row.get("created_at");
    let new_updated_at: chrono::DateTime<chrono::Utc> = updated_row.get("updated_at");

    assert_eq!(
        created_at, new_created_at,
        "created_at should not change on update"
    );
    assert!(
        new_updated_at > initial_updated_at,
        "updated_at should be greater after update"
    );
}

/// Test user retrieval by ID.
#[tokio::test]
#[serial]
#[ignore = "requires running PostgreSQL instance"]
async fn test_user_retrieval() {
    let pool = test_utils::setup_test_database().await;
    test_utils::cleanup_database(&pool).await;

    // Insert user
    let id: i32 = sqlx::query_scalar(
        "INSERT INTO users (name, email) 
         VALUES ($1, $2) 
         RETURNING id",
    )
    .bind("Jane Doe")
    .bind("jane@example.com")
    .fetch_one(&pool)
    .await
    .expect("Failed to insert user");

    // Retrieve user
    let row = sqlx::query("SELECT id, name, email FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("Failed to retrieve user");

    let retrieved_id: i32 = row.get("id");
    let name: String = row.get("name");
    let email: String = row.get("email");

    assert_eq!(retrieved_id, id);
    assert_eq!(name, "Jane Doe");
    assert_eq!(email, "jane@example.com");
}

/// Test user deletion.
#[tokio::test]
#[serial]
#[ignore = "requires running PostgreSQL instance"]
async fn test_user_deletion() {
    let pool = test_utils::setup_test_database().await;
    test_utils::cleanup_database(&pool).await;

    // Insert user
    let id: i32 = sqlx::query_scalar(
        "INSERT INTO users (name, email) 
         VALUES ($1, $2) 
         RETURNING id",
    )
    .bind("Delete Test")
    .bind("delete@example.com")
    .fetch_one(&pool)
    .await
    .expect("Failed to insert user");

    // Delete user
    let rows_affected = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .expect("Failed to delete user")
        .rows_affected();

    assert_eq!(rows_affected, 1, "Should delete exactly 1 row");

    // Verify user is gone
    let result = sqlx::query("SELECT id FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .expect("Failed to query for deleted user");

    assert!(result.is_none(), "User should not exist after deletion");
}

/// Test connection pool under concurrent load.
#[tokio::test]
#[serial]
#[ignore = "requires running PostgreSQL instance"]
async fn test_connection_pool_concurrent_operations() {
    let pool = test_utils::setup_test_database().await;
    test_utils::cleanup_database(&pool).await;

    // Spawn multiple concurrent insert operations
    let mut handles = vec![];
    for i in 0..10 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            sqlx::query(
                "INSERT INTO users (name, email) 
                 VALUES ($1, $2)",
            )
            .bind(format!("User {i}"))
            .bind(format!("user{i}@example.com"))
            .execute(&pool_clone)
            .await
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await.expect("Task panicked").expect("Insert failed");
    }

    // Verify all users were inserted
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("Failed to count users");

    assert_eq!(count, 10, "Should have inserted 10 users concurrently");
}

/// Test transaction isolation with the `test_utils` transaction helper.
#[tokio::test]
#[serial]
#[ignore = "requires running PostgreSQL instance"]
async fn test_transaction_isolation() {
    let pool = test_utils::setup_test_database().await;
    test_utils::cleanup_database(&pool).await;

    // Begin a transaction
    let mut tx = test_utils::transaction(&pool).await;

    // Insert within transaction
    sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
        .bind("Transaction Test")
        .bind("transaction@example.com")
        .execute(&mut *tx)
        .await
        .expect("Failed to insert in transaction");

    // Verify data exists within transaction
    let count_in_tx: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *tx)
        .await
        .expect("Failed to count in transaction");

    assert_eq!(count_in_tx, 1);

    // Rollback transaction
    tx.rollback().await.expect("Failed to rollback transaction");

    // Verify data was rolled back
    let count_after_rollback: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("Failed to count after rollback");

    assert_eq!(
        count_after_rollback, 0,
        "Data should be rolled back after transaction rollback"
    );
}

/// Test that connection pool is properly configured.
#[tokio::test]
#[serial]
#[ignore = "requires running PostgreSQL instance"]
async fn test_pool_configuration() {
    let pool = test_utils::setup_test_database().await;

    // Verify pool exists and can report size
    let _ = pool.size();

    // Test that we can acquire a connection
    let conn = pool
        .acquire()
        .await
        .expect("Should be able to acquire connection from pool");

    // Release connection
    drop(conn);
}
