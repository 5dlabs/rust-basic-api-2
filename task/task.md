# Task 2: Database Schema and Migrations

## Overview
Set up the PostgreSQL database schema with migrations using SQLx, establishing the foundation for data persistence in the REST API.

## Technical Requirements

### 1. Database Schema Design
Implement the following database schema:
- **Users table** with columns for id, name, email, and timestamps
- **Performance indexes** on email and created_at columns
- **Proper constraints** including primary key and unique constraints

### 2. Migration System
- Use SQLx migrations for schema versioning
- Create migration files in standard SQL format
- Enable automatic migration running on application startup

### 3. Connection Pool Management
- Configure connection pooling with appropriate limits
- Set timeout values for connection acquisition
- Implement graceful error handling for connection failures

## Implementation Guide

### Step 1: Install SQLx CLI
```bash
cargo install sqlx-cli --no-default-features --features postgres
```

### Step 2: Create Migrations Directory
```bash
mkdir migrations
```

### Step 3: Create Initial Migration
Create `migrations/001_initial_schema.sql`:

```sql
-- Main users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Performance indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_created_at ON users(created_at DESC);

-- Add trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at BEFORE UPDATE
    ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

### Step 4: Implement Database Connection Pool
Update `src/repository/mod.rs`:

```rust
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
}

#[cfg(test)]
pub mod test_utils;
```

### Step 5: Add Test Utilities
Create `src/repository/test_utils.rs`:

```rust
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Once;

static INIT: Once = Once::new();

pub async fn setup_test_database() -> PgPool {
    INIT.call_once(|| {
        dotenv::from_filename(".env.test").ok();
    });
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env.test");
        
    let pool = super::create_pool(&database_url).await
        .expect("Failed to create test database pool");
    
    // Run migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    pool
}

pub async fn transaction<'a>(pool: &'a PgPool) -> Transaction<'a, Postgres> {
    pool.begin().await
        .expect("Failed to begin transaction")
}

pub async fn cleanup_database(pool: &PgPool) {
    sqlx::query!("TRUNCATE TABLE users RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("Failed to cleanup database");
}
```

### Step 6: Update Main Application
Modify `src/main.rs` to include database initialization:

```rust
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ... existing initialization code ...
    
    // Create database pool
    let pool = repository::create_pool(&config.database_url)
        .await
        .context("Failed to create database pool")?;
    
    // Run migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .context("Failed to run database migrations")?;
    
    tracing::info!("Database connected and migrations completed");
    
    // Create app state
    let state = AppState { pool };
    
    // Build application router with state
    let app = Router::new()
        .route("/health", axum::routing::get(health_check))
        .with_state(state);
    
    // ... rest of server setup ...
}
```

### Step 7: Create Test Environment Configuration
Create `.env.test`:

```bash
DATABASE_URL=__AUTO_GENERATED__
TEST_DATABASE_URL=__AUTO_GENERATED__
DATABASE_USER=<test_user>
DATABASE_PASSWORD=<test_password>
DATABASE_HOST=localhost
DATABASE_PORT=5432
DATABASE_NAME=test_db
SERVER_PORT=3001
RUST_LOG=debug
```

## Database Operations

### Running Migrations Manually
```bash
# Run pending migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Show migration info
sqlx migrate info
```

### Creating New Migrations
```bash
# Create a new migration
sqlx migrate add <migration_name>
```

## Testing Strategy

### Integration Tests
Create `tests/database_integration.rs`:

```rust
use sqlx::PgPool;

#[sqlx::test]
async fn test_database_schema(pool: PgPool) {
    // Check if users table exists
    let result = sqlx::query!(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_name = 'users'
        )"
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert!(result.exists.unwrap());
}

#[sqlx::test]
async fn test_indexes_exist(pool: PgPool) {
    let indexes = sqlx::query!(
        "SELECT indexname FROM pg_indexes 
         WHERE tablename = 'users' 
         AND indexname IN ('idx_users_email', 'idx_users_created_at')"
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    
    assert_eq!(indexes.len(), 2);
}

#[sqlx::test]
async fn test_user_insertion(pool: PgPool) {
    let result = sqlx::query!(
        "INSERT INTO users (name, email) 
         VALUES ($1, $2) 
         RETURNING id",
        "Test User",
        "test@example.com"
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert!(result.id > 0);
}
```

## Dependencies and Prerequisites
- Task 1: Project Setup (must be completed first)
- PostgreSQL 12+ installed and running
- SQLx CLI tool installed
- Database created and accessible

## Related Tasks
- Task 3: API Server Implementation (depends on this task)
- Task 4: User Management API (depends on this task)
- Task 5: Data Validation (uses this schema)