# Task 4: User Repository Implementation - Autonomous Agent Prompt

You are a senior Rust developer specializing in database integration and repository patterns. Your task is to implement a clean, type-safe data access layer using SQLx and the repository pattern.

## Your Mission
Implement the UserRepository trait and its SQLx-based implementation to provide database operations for the User model in a Rust REST API.

## Context
- Tasks 2 and 3 have been completed (database setup and data models)
- You have access to a PostgreSQL database with a users table
- The User model and error handling are already implemented
- Focus on creating a clean abstraction for data access

## Required Implementations

### 1. Repository Trait (`src/repository/user_repository.rs`)
Define an async trait with these methods:
- `create_user`: Insert new user and return created entity
- `get_user`: Retrieve user by ID (returns Option)
- `get_users`: List all users
- `update_user`: Partial update (returns Option)
- `delete_user`: Remove user (returns success boolean)

### 2. SQLx Implementation
Create `SqlxUserRepository` that:
- Stores a PgPool for connection management
- Implements all trait methods using SQLx query macros
- Handles database errors appropriately
- Uses type-safe SQL queries

### 3. Dynamic Update Logic
For the update method:
- Check if user exists first
- Build SQL dynamically based on provided fields
- Only update fields that are Some(value)
- Always update the `updated_at` timestamp

## Technical Requirements
- Use `async-trait` crate for async trait support
- Use SQLx `query_as!` macro for compile-time SQL verification
- Map all database errors to ApiError::Database
- Return Option for queries that might not find results
- Use parameterized queries to prevent SQL injection

## SQL Query Specifications

### Insert Query
```sql
INSERT INTO users (name, email) 
VALUES ($1, $2) 
RETURNING id, name, email, created_at, updated_at
```

### Select Queries
- Single: `SELECT * FROM users WHERE id = $1`
- All: `SELECT * FROM users ORDER BY id`

### Update Query (Dynamic)
Build dynamically with:
- Base: `UPDATE users SET updated_at = NOW()`
- Add conditionally: `, name = $n` if name provided
- Add conditionally: `, email = $n` if email provided
- End with: `WHERE id = $n RETURNING *`

### Delete Query
```sql
DELETE FROM users WHERE id = $1
```

## Expected Deliverables
1. Complete implementation of UserRepository trait
2. SqlxUserRepository struct with all methods
3. Updated module exports in src/repository/mod.rs
4. Cargo.toml updated with async-trait dependency
5. All code compiles without warnings

## Quality Checklist
- [ ] All CRUD operations implemented
- [ ] Proper error handling with ApiError mapping
- [ ] No SQL injection vulnerabilities
- [ ] Efficient use of connection pool
- [ ] Dynamic update query works correctly
- [ ] Returns appropriate Option types
- [ ] No unwrap() or expect() calls
- [ ] Follows Rust naming conventions

## Implementation Steps
1. First, add async-trait to Cargo.toml
2. Create the trait definition with all required methods
3. Implement SqlxUserRepository with pool storage
4. Implement each CRUD method systematically
5. Pay special attention to the dynamic update logic
6. Update module exports to make types public

## Error Handling Guidelines
- All database errors should map to ApiError::Database
- Use `?` operator for error propagation
- Return None for get_user when not found (not an error)
- Return empty Vec for get_users when no users exist
- Log database errors before converting them

## Testing Approach
After implementation, verify:
- User creation returns valid ID
- Get operations handle existing and non-existing IDs
- Updates only modify specified fields
- Deletes return correct success status
- All timestamps are properly managed

## Important Notes
- The database schema already exists from Task 2
- Use NOW() for timestamp updates in PostgreSQL
- Remember that get_user returns Option (None is not an error)
- The update method should check existence before updating
- Keep the pool creation function in mod.rs intact

## Code Quality Standards
- Use clear, descriptive variable names
- Add comments for complex logic
- Keep methods focused and single-purpose
- Follow Rust idioms and best practices
- Ensure all public APIs are properly exported

Begin by verifying the existing repository module structure, then implement the trait and its SQLx-based implementation. Test each method to ensure it works correctly with the database.