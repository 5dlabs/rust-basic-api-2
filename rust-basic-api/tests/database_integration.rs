use rust_basic_api::config::database_url_from_env;
use rust_basic_api::repository::create_pool;
use serial_test::serial;
use sqlx::{
    migrate::Migrator,
    types::chrono::{DateTime, Utc},
    PgPool, Row,
};
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
    sync::Once,
    time::Duration,
};
use tokio::time::sleep;

static INIT_ENV: Once = Once::new();
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

fn ensure_test_env() {
    INIT_ENV.call_once(|| {
        if let Ok(file) = File::open(".env.test") {
            for line in BufReader::new(file)
                .lines()
                .map_while(std::result::Result::ok)
            {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }

                let Some((key, value)) = trimmed.split_once('=') else {
                    continue;
                };

                if env::var_os(key).is_none() {
                    env::set_var(key, value);
                }
            }
        }
    });
}

async fn setup_pool() -> PgPool {
    ensure_test_env();

    let database_url = database_url_from_env().expect(
        "configure TEST_DATABASE_URL, DATABASE_URL, or component variables for integration tests",
    );

    let pool = create_pool(&database_url)
        .await
        .expect("failed to create PostgreSQL pool");

    sqlx::query("DROP TRIGGER IF EXISTS update_users_updated_at ON users")
        .execute(&pool)
        .await
        .expect("failed to drop existing trigger before migrations");
    sqlx::query("DROP FUNCTION IF EXISTS update_updated_at_column")
        .execute(&pool)
        .await
        .expect("failed to drop trigger function before migrations");
    sqlx::query("DROP TABLE IF EXISTS users CASCADE")
        .execute(&pool)
        .await
        .expect("failed to drop users table before migrations");
    sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations CASCADE")
        .execute(&pool)
        .await
        .expect("failed to reset migrations history before migrations");

    assert!(
        !MIGRATOR.migrations.is_empty(),
        "expected at least one migration to be embedded"
    );

    MIGRATOR
        .run(&pool)
        .await
        .expect("failed to run database migrations");

    pool
}

async fn cleanup(pool: &PgPool) {
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
    .expect("failed to truncate users table");
}

#[tokio::test]
#[serial]
async fn test_users_table_exists() {
    let pool = setup_pool().await;

    let row = sqlx::query("SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'users') AS present")
        .fetch_one(&pool)
        .await
        .expect("failed to query information_schema");

    let exists: bool = row.get("present");
    assert!(exists, "users table should exist after migrations");

    cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_indexes_exist() {
    let pool = setup_pool().await;

    let indexes = sqlx::query(
        "SELECT indexname FROM pg_indexes WHERE tablename = 'users' AND indexname IN ('idx_users_email', 'idx_users_created_at')",
    )
    .fetch_all(&pool)
    .await
    .expect("failed to fetch index metadata");

    assert_eq!(
        indexes.len(),
        2,
        "expected both email and created_at indexes to exist"
    );

    cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_user_insertion_and_defaults() {
    let pool = setup_pool().await;

    let row = sqlx::query(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, created_at, updated_at",
    )
    .bind("Test User")
    .bind("integration@example.com")
    .fetch_one(&pool)
    .await
    .expect("failed to insert user");

    let id: i32 = row.get("id");
    let created: DateTime<Utc> = row.get("created_at");
    let updated: DateTime<Utc> = row.get("updated_at");

    assert!(id > 0, "user id should auto-increment");
    assert!(
        updated >= created,
        "updated_at should be initialized to created_at"
    );

    cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_email_unique_constraint() {
    let pool = setup_pool().await;

    sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
        .bind("First User")
        .bind("duplicate@example.com")
        .execute(&pool)
        .await
        .expect("initial insert should succeed");

    let result = sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
        .bind("Second User")
        .bind("duplicate@example.com")
        .execute(&pool)
        .await;

    assert!(
        result.is_err(),
        "second insert should violate unique constraint"
    );

    cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_updated_at_trigger() {
    let pool = setup_pool().await;

    let row = sqlx::query(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, created_at, updated_at",
    )
    .bind("Trigger User")
    .bind("trigger@example.com")
    .fetch_one(&pool)
    .await
    .expect("failed to insert user for trigger test");

    let user_id: i32 = row.get("id");
    let created_at: DateTime<Utc> = row.get("created_at");

    sleep(Duration::from_millis(50)).await;

    sqlx::query("UPDATE users SET name = $1 WHERE id = $2")
        .bind("Updated User")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("failed to update user");

    let row = sqlx::query("SELECT created_at, updated_at FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("failed to fetch updated user");

    let updated_at: DateTime<Utc> = row.get("updated_at");

    assert!(
        updated_at > created_at,
        "updated_at should be greater than created_at after update"
    );

    cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_connection_pool_handles_multiple_queries() {
    let pool = setup_pool().await;

    let mut handles = Vec::new();
    for _ in 0..5 {
        let pool_clone = pool.clone();
        handles.push(tokio::spawn(async move {
            sqlx::query("SELECT 1")
                .fetch_one(&pool_clone)
                .await
                .expect("heartbeat query should succeed");
        }));
    }

    for handle in handles {
        handle.await.expect("task should complete successfully");
    }

    cleanup(&pool).await;
}
