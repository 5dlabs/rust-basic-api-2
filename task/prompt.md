# Autonomous Agent Prompt: Database Schema and Migrations

## Task Objective
Set up PostgreSQL database schema with SQLx migrations for a REST API application. Create the users table with appropriate indexes and implement connection pooling for efficient database access.

## Requirements

### 1. Migration System Setup
- Create migrations directory in project root
- Implement initial schema migration with users table
- Configure SQLx to run migrations automatically

### 2. Database Schema
Create the following schema:
- **users** table with:
  - `id` (SERIAL PRIMARY KEY)
  - `name` (VARCHAR(255) NOT NULL)
  - `email` (VARCHAR(255) UNIQUE NOT NULL)
  - `created_at` (TIMESTAMP WITH TIME ZONE)
  - `updated_at` (TIMESTAMP WITH TIME ZONE)
- Indexes on `email` and `created_at` columns
- Trigger for automatic `updated_at` updates

### 3. Connection Pool Implementation
- Configure PostgreSQL connection pool
- Set appropriate connection limits and timeouts
- Implement error handling for connection failures

### 4. Application Integration
- Update main application to initialize database pool
- Run migrations on application startup
- Pass database pool through application state

### 5. Testing Infrastructure
- Create test utilities for database testing
- Implement test database setup and teardown
- Add integration tests for schema validation

## Execution Steps

1. **Install SQLx CLI**
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```

2. **Create Migrations Directory**
   ```bash
   mkdir migrations
   ```

3. **Create Initial Migration**
   - File: `migrations/001_initial_schema.sql`
   - Include users table definition
   - Add performance indexes
   - Create updated_at trigger

4. **Implement Connection Pool**
   - Update `src/repository/mod.rs`
   - Configure pool with max/min connections
   - Set timeout values

5. **Create Test Utilities**
   - File: `src/repository/test_utils.rs`
   - Implement test database setup
   - Add transaction helper for test isolation
   - Include cleanup functions

6. **Update Main Application**
   - Add database pool initialization
   - Run migrations on startup
   - Create AppState struct with pool
   - Pass state to router

7. **Configure Test Environment**
   - Create `.env.test` file
   - Set test database connection string using environment variable overrides, for example:

   ```bash
   DATABASE_URL=postgresql://${TEST_DB_USER:-postgres}:${TEST_DB_PASSWORD:-postgres}@localhost:5432/rust_basic_api_test
   TEST_DATABASE_URL=${DATABASE_URL}
   ```

8. **Add Integration Tests**
   - Verify table creation
   - Check index existence
   - Test basic CRUD operations

## Validation

### Manual Testing
1. Run migrations: `sqlx migrate run`
2. Check migration status: `sqlx migrate info`
3. Start application and verify database connection
4. Connect to database and verify schema:
   ```sql
   \d users
   \di
   ```

### Automated Testing
1. Run integration tests: `cargo test --test database_integration`
2. Verify all tests pass
3. Check test database cleanup

### Performance Validation
1. Monitor connection pool metrics
2. Test concurrent connections
3. Verify connection timeout behavior

## Expected Outcome

A fully configured database layer with:
- Users table created with proper schema
- Performance indexes in place
- Automatic updated_at trigger working
- Connection pool configured and operational
- Migrations running automatically on startup
- Test infrastructure ready for integration testing
- Application state containing database pool

## Error Handling

### Common Issues and Solutions

1. **Database Connection Failed**
   - Verify PostgreSQL is running
   - Check DATABASE_URL format
   - Ensure database exists
   - Verify network connectivity

2. **Migration Failed**
   - Check SQL syntax
   - Verify database permissions
   - Ensure migrations directory exists
   - Check for conflicting migrations

3. **Pool Exhaustion**
   - Increase max_connections
   - Check for connection leaks
   - Review query performance
   - Implement connection recycling

## Best Practices

- Always use prepared statements
- Implement proper connection lifecycle management
- Use transactions for multi-step operations
- Add appropriate indexes for query patterns
- Monitor database performance metrics
- Keep migrations idempotent when possible
- Version control all migration files
- Test migrations in development first

## Notes

- Ensure PostgreSQL version compatibility (12+)
- Use environment variables for all configuration
- Never commit .env files with real credentials
- Consider using migration checksums for production
- Document any manual database changes
- Keep test and production schemas in sync