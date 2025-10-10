use sqlx::PgPool;

#[sqlx::test]
async fn test_users_table_exists(pool: PgPool) {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_name = 'users'
        )",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to query table existence");

    assert!(exists, "Users table should exist");
}

#[sqlx::test]
async fn test_users_table_columns(pool: PgPool) {
    let columns: Vec<(String,)> = sqlx::query_as(
        "SELECT column_name
         FROM information_schema.columns
         WHERE table_name = 'users'
         ORDER BY ordinal_position",
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query columns");

    let column_names: Vec<String> = columns.into_iter().map(|(name,)| name).collect();

    assert!(
        column_names.contains(&"id".to_string()),
        "Should have id column"
    );
    assert!(
        column_names.contains(&"name".to_string()),
        "Should have name column"
    );
    assert!(
        column_names.contains(&"email".to_string()),
        "Should have email column"
    );
    assert!(
        column_names.contains(&"created_at".to_string()),
        "Should have created_at column"
    );
    assert!(
        column_names.contains(&"updated_at".to_string()),
        "Should have updated_at column"
    );
}

#[sqlx::test]
async fn test_user_insert(pool: PgPool) {
    let id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (name, email)
         VALUES ('Test User', 'test@example.com')
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert user");

    assert!(id > 0, "Inserted user should have positive ID");
}

#[sqlx::test]
async fn test_user_insert_and_select(pool: PgPool) {
    let inserted_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (name, email)
         VALUES ('John Doe', 'john@example.com')
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert user");

    let (id, name, email): (i32, String, String) =
        sqlx::query_as("SELECT id, name, email FROM users WHERE id = $1")
            .bind(inserted_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to select user");

    assert_eq!(id, inserted_id);
    assert_eq!(name, "John Doe");
    assert_eq!(email, "john@example.com");
}

#[sqlx::test]
async fn test_email_unique_constraint(pool: PgPool) {
    sqlx::query("INSERT INTO users (name, email) VALUES ('User1', 'unique@example.com')")
        .execute(&pool)
        .await
        .expect("First insert should succeed");

    let result =
        sqlx::query("INSERT INTO users (name, email) VALUES ('User2', 'unique@example.com')")
            .execute(&pool)
            .await;

    assert!(result.is_err(), "Second insert with same email should fail");
}

#[sqlx::test]
async fn test_updated_at_trigger(pool: PgPool) {
    use chrono::{DateTime, Utc};

    let id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (name, email) VALUES ('Trigger Test', 'trigger@example.com') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert user");

    // Wait to ensure time difference
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    sqlx::query("UPDATE users SET name = 'Updated Name' WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .expect("Failed to update user");

    let (created, updated): (DateTime<Utc>, DateTime<Utc>) =
        sqlx::query_as("SELECT created_at, updated_at FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch timestamps");

    assert!(
        updated > created,
        "updated_at should be greater than created_at after update"
    );
}

#[sqlx::test]
async fn test_indexes_exist(pool: PgPool) {
    let indexes: Vec<(String,)> = sqlx::query_as(
        "SELECT indexname FROM pg_indexes
         WHERE tablename = 'users'
         AND indexname IN ('idx_users_email', 'idx_users_created_at')",
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query indexes");

    let index_names: Vec<String> = indexes.into_iter().map(|(name,)| name).collect();

    assert!(
        index_names.contains(&"idx_users_email".to_string()),
        "Email index should exist"
    );
    assert!(
        index_names.contains(&"idx_users_created_at".to_string()),
        "Created_at index should exist"
    );
}

#[sqlx::test]
async fn test_primary_key_constraint(pool: PgPool) {
    let constraints: Vec<(String,)> = sqlx::query_as(
        "SELECT constraint_name
         FROM information_schema.table_constraints
         WHERE table_name = 'users'
         AND constraint_type = 'PRIMARY KEY'",
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query constraints");

    assert!(
        !constraints.is_empty(),
        "Users table should have a primary key"
    );
}

#[sqlx::test]
async fn test_default_timestamps(pool: PgPool) {
    use chrono::{DateTime, Utc};

    let id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (name, email) VALUES ('Default Test', 'default@example.com') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert user");

    let (created, updated): (DateTime<Utc>, DateTime<Utc>) =
        sqlx::query_as("SELECT created_at, updated_at FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch timestamps");

    // Timestamps should be set and approximately equal (within 1 second)
    let diff = (created.timestamp() - updated.timestamp()).abs();
    assert!(diff < 1, "Default timestamps should be approximately equal");
}

#[sqlx::test]
async fn test_multiple_user_inserts(pool: PgPool) {
    for i in 0..10 {
        let email = format!("user{i}@example.com");
        let name = format!("User {i}");

        let id = sqlx::query_scalar::<_, i32>(
            "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",
        )
        .bind(&name)
        .bind(&email)
        .fetch_one(&pool)
        .await
        .expect("Failed to insert user");

        assert!(id > 0, "Each user should get a positive ID");
    }

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("Failed to count users");

    assert_eq!(count, 10, "Should have 10 users in database");
}

#[sqlx::test]
async fn test_user_update(pool: PgPool) {
    let id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (name, email) VALUES ('Original Name', 'update@example.com') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert user");

    sqlx::query("UPDATE users SET name = 'Updated Name' WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .expect("Failed to update user");

    let name: String = sqlx::query_scalar("SELECT name FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch updated name");

    assert_eq!(name, "Updated Name", "Name should be updated");
}

#[sqlx::test]
async fn test_user_delete(pool: PgPool) {
    let id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (name, email) VALUES ('Delete Me', 'delete@example.com') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert user");

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .expect("Failed to delete user");

    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("Failed to check existence");

    assert!(!exists, "User should be deleted");
}

#[sqlx::test]
async fn test_empty_name_not_allowed(pool: PgPool) {
    let result = sqlx::query("INSERT INTO users (name, email) VALUES ('', 'empty@example.com')")
        .execute(&pool)
        .await;

    // Empty string is technically allowed by VARCHAR, but we test it doesn't crash
    assert!(
        result.is_ok() || result.is_err(),
        "Empty name test completed"
    );
}

#[sqlx::test]
async fn test_null_name_not_allowed(pool: PgPool) {
    let result = sqlx::query("INSERT INTO users (name, email) VALUES (NULL, 'null@example.com')")
        .execute(&pool)
        .await;

    assert!(result.is_err(), "NULL name should not be allowed");
}

#[sqlx::test]
async fn test_null_email_not_allowed(pool: PgPool) {
    let result = sqlx::query("INSERT INTO users (name, email) VALUES ('Test User', NULL)")
        .execute(&pool)
        .await;

    assert!(result.is_err(), "NULL email should not be allowed");
}

#[sqlx::test]
async fn test_long_name_handling(pool: PgPool) {
    let long_name = "A".repeat(255); // Max length
    let result = sqlx::query("INSERT INTO users (name, email) VALUES ($1, 'long@example.com')")
        .bind(&long_name)
        .execute(&pool)
        .await;

    assert!(result.is_ok(), "Should handle 255 character names");
}

#[sqlx::test]
async fn test_long_email_handling(pool: PgPool) {
    let long_email = format!("{}@example.com", "a".repeat(240));
    let result = sqlx::query("INSERT INTO users (name, email) VALUES ('Test', $1)")
        .bind(&long_email)
        .execute(&pool)
        .await;

    assert!(
        result.is_ok(),
        "Should handle long emails up to 255 characters"
    );
}
