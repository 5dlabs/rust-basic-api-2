use std::collections::HashSet;
use std::time::Duration;

use sqlx::types::chrono::Utc;
use sqlx::{PgPool, Row};

#[sqlx::test]
async fn test_users_table_exists(pool: PgPool) {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' AND table_name = 'users'
        )",
    )
    .fetch_one(&pool)
    .await
    .expect("failed to check users table existence");

    assert!(exists, "users table should exist after migrations");
}

#[sqlx::test]
async fn test_indexes_exist(pool: PgPool) {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let indexes = sqlx::query(
        "SELECT indexname FROM pg_indexes \
         WHERE schemaname = 'public' AND tablename = 'users'",
    )
    .fetch_all(&pool)
    .await
    .expect("failed to fetch users indexes");

    let index_names: HashSet<String> = indexes
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("indexname").ok())
        .collect();

    assert!(index_names.contains("users_pkey"));
    assert!(index_names.contains("idx_users_email"));
    assert!(index_names.contains("idx_users_created_at"));
}

#[sqlx::test]
async fn test_user_insertion(pool: PgPool) {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",
    )
    .bind("Test User")
    .bind("user_insertion@example.com")
    .fetch_one(&pool)
    .await
    .expect("failed to insert user");

    assert!(id > 0, "inserted user id should be positive");
}

#[sqlx::test]
async fn test_email_unique_constraint(pool: PgPool) {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");

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

    match result {
        Err(sqlx::Error::Database(db_error)) => {
            let constraint = db_error.constraint();
            assert_eq!(constraint, Some("users_email_key"));
        }
        other => panic!("expected database unique violation, got {other:?}"),
    }
}

#[sqlx::test]
async fn test_updated_at_trigger(pool: PgPool) {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let row = sqlx::query(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, created_at, updated_at",
    )
    .bind("Trigger User")
    .bind("trigger@example.com")
    .fetch_one(&pool)
    .await
    .expect("failed to insert user for trigger test");

    let id: i32 = row.try_get("id").expect("missing id");
    let created_at: sqlx::types::chrono::DateTime<Utc> =
        row.try_get("created_at").expect("missing created_at");
    let initial_updated_at: sqlx::types::chrono::DateTime<Utc> =
        row.try_get("updated_at").expect("missing updated_at");

    assert_eq!(
        created_at, initial_updated_at,
        "timestamps should match on insert"
    );

    tokio::time::sleep(Duration::from_millis(150)).await;

    sqlx::query("UPDATE users SET name = $1 WHERE id = $2")
        .bind("Trigger Updated")
        .bind(id)
        .execute(&pool)
        .await
        .expect("failed to update user for trigger test");

    let row = sqlx::query("SELECT created_at, updated_at FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("failed to fetch updated user");

    let created: sqlx::types::chrono::DateTime<Utc> = row
        .try_get("created_at")
        .expect("missing created_at column");
    let updated: sqlx::types::chrono::DateTime<Utc> = row
        .try_get("updated_at")
        .expect("missing updated_at column");

    assert!(
        updated > created,
        "updated_at should be later than created_at"
    );
}
