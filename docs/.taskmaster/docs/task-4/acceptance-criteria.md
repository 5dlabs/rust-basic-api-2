# Task 4: User Repository Implementation - Acceptance Criteria

## Definition of Done
This task is considered complete when all the following criteria are met:

## Core Requirements

### 1. Repository Trait Definition
- [ ] `src/repository/user_repository.rs` file exists and compiles
- [ ] UserRepository trait defined with async_trait macro
- [ ] All five CRUD methods declared in trait
- [ ] Methods use appropriate return types (Result, Option)
- [ ] Trait is public and properly exported

### 2. SQLx Implementation
- [ ] SqlxUserRepository struct defined with PgPool field
- [ ] Constructor method `new(pool: PgPool)` implemented
- [ ] All trait methods implemented for SqlxUserRepository
- [ ] Uses SQLx query macros for type safety
- [ ] Proper error conversion to ApiError

### 3. CRUD Operations
- [ ] `create_user` inserts and returns created user with ID
- [ ] `get_user` returns Option<User> for given ID
- [ ] `get_users` returns Vec<User> ordered by ID
- [ ] `update_user` performs partial updates correctly
- [ ] `delete_user` removes user and returns success status

### 4. Dynamic Update Implementation
- [ ] Checks if user exists before attempting update
- [ ] Builds SQL query dynamically based on provided fields
- [ ] Only updates fields that are Some(value)
- [ ] Always updates the `updated_at` timestamp
- [ ] Returns None if user doesn't exist

### 5. Module Organization
- [ ] `src/repository/mod.rs` updated with exports
- [ ] UserRepository trait is publicly exported
- [ ] SqlxUserRepository is publicly exported
- [ ] Existing pool creation function preserved

### 6. Dependencies
- [ ] Cargo.toml includes `async-trait = "0.1"`
- [ ] Project compiles with new dependency

## Functional Test Cases

### Test Case 1: Create User
```rust
// Given
let repo = SqlxUserRepository::new(pool);
let req = CreateUserRequest {
    name: "John Doe".to_string(),
    email: "john@example.com".to_string(),
};

// When
let user = repo.create_user(req).await?;

// Then
assert!(user.id > 0);
assert_eq!(user.name, "John Doe");
assert_eq!(user.email, "john@example.com");
assert!(user.created_at <= user.updated_at);
```

### Test Case 2: Get Existing User
```rust
// Given
let created = repo.create_user(req).await?;

// When
let user = repo.get_user(created.id).await?;

// Then
assert!(user.is_some());
assert_eq!(user.unwrap().id, created.id);
```

### Test Case 3: Get Non-Existent User
```rust
// When
let user = repo.get_user(99999).await?;

// Then
assert!(user.is_none());
```

### Test Case 4: Partial Update
```rust
// Given
let created = repo.create_user(initial_req).await?;
let update_req = UpdateUserRequest {
    name: Some("New Name".to_string()),
    email: None,
};

// When
let updated = repo.update_user(created.id, update_req).await?;

// Then
assert!(updated.is_some());
let user = updated.unwrap();
assert_eq!(user.name, "New Name");
assert_eq!(user.email, created.email); // Unchanged
assert!(user.updated_at > created.updated_at);
```

### Test Case 5: Delete User
```rust
// Given
let created = repo.create_user(req).await?;

// When
let deleted = repo.delete_user(created.id).await?;

// Then
assert!(deleted);
let user = repo.get_user(created.id).await?;
assert!(user.is_none());
```

### Test Case 6: List All Users
```rust
// Given
repo.create_user(req1).await?;
repo.create_user(req2).await?;

// When
let users = repo.get_users().await?;

// Then
assert!(users.len() >= 2);
assert!(users.windows(2).all(|w| w[0].id < w[1].id)); // Ordered by ID
```

## Non-Functional Requirements

### Performance
- [ ] Connection pool used efficiently (no connection leaks)
- [ ] Queries execute in reasonable time (<100ms for single operations)
- [ ] No N+1 query problems
- [ ] Batch operations use single queries where possible

### Security
- [ ] All queries use parameterized statements
- [ ] No SQL injection vulnerabilities
- [ ] Database errors don't expose connection details
- [ ] User input properly sanitized

### Code Quality
- [ ] No compiler warnings
- [ ] No use of unwrap() or expect()
- [ ] Consistent error handling throughout
- [ ] Clear separation between trait and implementation
- [ ] Follows Rust naming conventions

### Error Handling
- [ ] Database errors properly mapped to ApiError
- [ ] Connection failures handled gracefully
- [ ] Constraint violations (e.g., duplicate email) handled
- [ ] Transaction rollbacks work correctly

## SQL Query Verification

### Create Query
- [ ] Uses RETURNING clause for efficiency
- [ ] Returns all user fields including timestamps
- [ ] Handles auto-generated ID correctly

### Update Query
- [ ] Only includes fields that need updating
- [ ] Always updates `updated_at` timestamp
- [ ] Uses RETURNING clause to get updated data
- [ ] Handles empty update requests appropriately

### Delete Query
- [ ] Returns rows_affected for success detection
- [ ] Handles cascade deletes if configured
- [ ] Doesn't error on non-existent ID

## Integration Requirements
- [ ] Works with existing User model from Task 3
- [ ] Uses ApiError from Task 3
- [ ] Compatible with database schema from Task 2
- [ ] Can be injected into handlers (Task 5)

## Database Constraints
- [ ] Respects unique constraint on email
- [ ] Handles foreign key constraints properly
- [ ] Manages NOT NULL constraints correctly
- [ ] Timestamp fields auto-populate correctly

## Testing Requirements
- [ ] Integration tests can run against test database
- [ ] Tests use transactions for isolation
- [ ] Test data cleanup happens automatically
- [ ] All CRUD operations have test coverage

## Documentation
- [ ] Public methods have doc comments
- [ ] Complex logic includes inline comments
- [ ] SQL queries documented for clarity
- [ ] Module-level documentation explains pattern

## Verification Steps
1. Run `cargo build` - compiles without errors
2. Run `cargo check` - no warnings
3. Run `cargo test` - repository tests pass
4. Verify database operations with test data
5. Check for SQL injection vulnerabilities
6. Verify connection pool behavior

## Performance Benchmarks
- [ ] Single user fetch: <10ms
- [ ] Create user: <20ms
- [ ] Update user: <15ms
- [ ] Delete user: <10ms
- [ ] List 100 users: <50ms

## Rollback Plan
If issues are found:
1. Identify specific failing operations
2. Check database logs for errors
3. Verify connection pool configuration
4. Review SQL query syntax
5. Rollback to previous working version if critical

## Sign-off Checklist
- [ ] Developer: Implementation complete and tested
- [ ] Database: Queries optimized and indexed
- [ ] Security: No SQL injection risks
- [ ] Performance: Meets benchmark requirements
- [ ] Integration: Works with existing code