# Task 4: User Repository Implementation

## Overview
Implement the UserRepository trait and SQLx-based implementation for database operations. This task establishes the data access layer that provides an abstraction over database operations, following the repository pattern for clean architecture.

## Dependencies
- Task 2: Database setup and connection (pool creation)
- Task 3: Data models and error handling (User model, error types)

## Technical Specifications

### 1. Repository Pattern
Define a trait-based abstraction for data access:
- **Trait Definition**: Async trait for database operations
- **Implementation**: SQLx-based concrete implementation
- **Connection Management**: Use connection pool for efficiency
- **Error Handling**: Proper error mapping and propagation

### 2. CRUD Operations
Implement complete CRUD functionality:
- **Create**: Insert new users with auto-generated timestamps
- **Read**: Get single user by ID or all users
- **Update**: Partial updates with dynamic query building
- **Delete**: Remove users and return success status

### 3. Database Interaction
Use SQLx for type-safe database operations:
- **Query Macros**: Use `query_as!` for compile-time SQL verification
- **Type Safety**: Automatic mapping between SQL and Rust types
- **Transaction Support**: Enable transaction-based operations
- **Connection Pooling**: Efficient resource management

## Implementation Guide

### Step 1: Add Dependencies
Update `Cargo.toml`:
```toml
async-trait = "0.1"
```

### Step 2: Define Repository Trait
1. Create `src/repository/user_repository.rs`
2. Define `UserRepository` trait with async methods
3. Use `async_trait` macro for async trait support

### Step 3: Implement SQLx Repository
1. Create `SqlxUserRepository` struct
2. Store `PgPool` for database connections
3. Implement all CRUD operations
4. Handle database errors appropriately

### Step 4: Dynamic Update Query
Build update queries dynamically:
1. Check if user exists before updating
2. Build SQL based on provided fields
3. Use parameterized queries for safety
4. Return updated user or None

### Step 5: Module Organization
1. Update `src/repository/mod.rs`
2. Export public types
3. Maintain existing pool creation function

## Code Structure

```
src/repository/
├── mod.rs                  # Module exports and pool creation
├── user_repository.rs      # Trait and implementation
└── user_repository_test.rs # Integration tests
```

## Key Design Decisions

### Repository Pattern Benefits
- **Abstraction**: Decouple business logic from data access
- **Testability**: Easy to mock for unit tests
- **Flexibility**: Switch database implementations easily
- **Type Safety**: Compile-time SQL verification with SQLx

### Async Operations
- All methods are async for non-blocking I/O
- Use `async_trait` for trait support
- Proper error propagation with `?` operator

### Error Handling Strategy
- Map SQLx errors to ApiError
- Return Option for queries that may not find results
- Provide meaningful error messages
- Log database errors for debugging

## SQL Queries

### Create User
```sql
INSERT INTO users (name, email) 
VALUES ($1, $2) 
RETURNING id, name, email, created_at, updated_at
```

### Get User by ID
```sql
SELECT id, name, email, created_at, updated_at 
FROM users 
WHERE id = $1
```

### Update User (Dynamic)
```sql
UPDATE users 
SET updated_at = NOW(), 
    name = $2,  -- if provided
    email = $3  -- if provided
WHERE id = $1 
RETURNING id, name, email, created_at, updated_at
```

## Testing Strategy

### Integration Tests
1. **Create User**: Verify insertion and ID generation
2. **Get User**: Test retrieval by ID
3. **Update User**: Test partial updates
4. **Delete User**: Verify deletion and cascade effects
5. **List Users**: Test pagination and ordering

### Transaction Tests
- Test rollback behavior
- Verify isolation levels
- Check concurrent access handling

### Error Cases
- Duplicate email handling
- Non-existent user operations
- Database connection failures
- Invalid data types

## Performance Considerations

### Connection Pooling
- Reuse database connections
- Configure pool size appropriately
- Monitor connection usage

### Query Optimization
- Use indexes on frequently queried fields
- Avoid N+1 query problems
- Consider pagination for large result sets

### Caching Strategy
- Consider caching frequently accessed users
- Invalidate cache on updates
- Use appropriate TTL values

## Success Criteria
- All CRUD operations work correctly
- SQL queries are type-safe and verified at compile time
- Proper error handling and logging
- Integration tests pass
- No SQL injection vulnerabilities
- Efficient use of database connections

## Best Practices
- Use parameterized queries to prevent SQL injection
- Always use transactions for multi-step operations
- Log slow queries for performance monitoring
- Keep repository methods focused and single-purpose
- Use meaningful variable and function names

## Common Pitfalls to Avoid
- Don't expose SQL errors directly to users
- Avoid building SQL strings with string concatenation
- Don't forget to handle None cases for optional queries
- Remember to update `updated_at` on modifications
- Don't leak database connections

## Future Enhancements
- Add pagination support for list operations
- Implement soft deletes
- Add query filtering and sorting
- Support bulk operations
- Add caching layer

## Related Documentation
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Async Trait Documentation](https://docs.rs/async-trait/)
- [Repository Pattern](https://martinfowler.com/eaaCatalog/repository.html)
- [PostgreSQL Best Practices](https://wiki.postgresql.org/wiki/Main_Page)