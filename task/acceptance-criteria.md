# Acceptance Criteria: Database Schema and Migrations

## Required Deliverables

### 1. Migration Files
- [ ] `migrations/` directory exists in project root
- [ ] `migrations/001_initial_schema.sql` exists with:
  - [ ] Complete users table definition
  - [ ] All required columns with correct types
  - [ ] Primary key constraint on id
  - [ ] Unique constraint on email
  - [ ] Default values for timestamps
  - [ ] Index on email column
  - [ ] Index on created_at column
  - [ ] Updated_at trigger function
  - [ ] Trigger on users table

### 2. Repository Module Updates
- [ ] `src/repository/mod.rs` contains:
  - [ ] `create_pool()` function
  - [ ] Pool configuration with max_connections
  - [ ] Connection timeout settings
  - [ ] Proper error handling
  - [ ] Test module declaration (when cfg(test))
- [ ] `src/repository/test_utils.rs` exists with:
  - [ ] `setup_test_database()` function
  - [ ] `transaction()` helper function
  - [ ] `cleanup_database()` function
  - [ ] Proper test initialization with Once

### 3. Main Application Updates
- [ ] `src/main.rs` includes:
  - [ ] Database pool creation
  - [ ] Migration execution on startup
  - [ ] AppState struct with pool field
  - [ ] State passed to router
  - [ ] Error handling with context
  - [ ] Logging for database operations

### 4. Configuration Files
- [ ] `.env.test` exists with:
  - [ ] TEST_DATABASE_URL configuration
  - [ ] Test-specific port configuration
  - [ ] Debug logging level

## Functional Tests

### 1. Migration Execution
```bash
sqlx migrate run
```
**Expected**: 
- Migrations complete successfully
- No SQL errors
- Migration history updated

### 2. Migration Verification
```bash
sqlx migrate info
```
**Expected**: Shows 001_initial_schema as applied

### 3. Database Schema Verification
```sql
-- Connect to database and run:
\d users
```
**Expected**:
- Table structure matches specification
- All columns present with correct types
- Constraints properly defined

### 4. Index Verification
```sql
\di
```
**Expected**:
- idx_users_email exists
- idx_users_created_at exists
- Primary key index exists

### 5. Application Startup Test
```bash
cargo run
```
**Expected**:
- Application starts without database errors
- Log shows "Database connected and migrations completed"
- Health endpoint still works

### 6. Connection Pool Test
```bash
# Start application and monitor logs
cargo run
# In another terminal, stress test with multiple connections
for i in {1..20}; do curl http://localhost:3000/health & done
```
**Expected**: All requests succeed without connection errors

### 7. Integration Test Execution
```bash
cargo test --test database_integration
```
**Expected**: All tests pass

## Database Tests

### 1. Table Existence Test
```rust
#[sqlx::test]
async fn test_users_table_exists(pool: PgPool) {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_name = 'users'
        )"
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert!(exists);
}
```
**Expected**: Test passes

### 2. Insert Operation Test
```rust
#[sqlx::test]
async fn test_user_insert(pool: PgPool) {
    let id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (name, email) 
         VALUES ('Test', 'test@example.com') 
         RETURNING id"
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert!(id > 0);
}
```
**Expected**: Test passes with valid ID returned

### 3. Unique Constraint Test
```rust
#[sqlx::test]
async fn test_email_unique_constraint(pool: PgPool) {
    sqlx::query("INSERT INTO users (name, email) VALUES ('User1', 'same@example.com')")
        .execute(&pool)
        .await
        .unwrap();
    
    let result = sqlx::query("INSERT INTO users (name, email) VALUES ('User2', 'same@example.com')")
        .execute(&pool)
        .await;
    
    assert!(result.is_err());
}
```
**Expected**: Second insert fails with unique violation

### 4. Updated_at Trigger Test
```rust
#[sqlx::test]
async fn test_updated_at_trigger(pool: PgPool) {
    let id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (name, email) VALUES ('Test', 'trigger@example.com') RETURNING id"
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    sqlx::query("UPDATE users SET name = 'Updated' WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    
    let (created, updated) = sqlx::query_as::<_, (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        "SELECT created_at, updated_at FROM users WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert!(updated > created);
}
```
**Expected**: updated_at is later than created_at

## Non-Functional Requirements

### Performance
- [ ] Connection pool initializes in < 5 seconds
- [ ] Migrations complete in < 2 seconds
- [ ] Database queries respond in < 100ms
- [ ] Pool handles 10+ concurrent connections

### Reliability
- [ ] Graceful handling of database unavailability
- [ ] Automatic reconnection on connection loss
- [ ] Transaction rollback on errors
- [ ] No connection leaks under load

### Security
- [ ] No SQL injection vulnerabilities
- [ ] Prepared statements used throughout
- [ ] Sensitive data not logged
- [ ] Test credentials separate from production

## Definition of Done

1. All migration files created and valid
2. Database pool implementation complete
3. Migrations run successfully
4. Application integrates with database
5. All schema elements verified in database
6. Integration tests pass
7. Connection pooling works under load
8. Error handling implemented throughout
9. Test utilities functional
10. Documentation complete