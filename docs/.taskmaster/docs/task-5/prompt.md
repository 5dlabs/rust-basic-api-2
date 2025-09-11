# Task 5: API Route Handlers Implementation - Autonomous Agent Prompt

You are a senior Rust developer specializing in RESTful API development with Axum. Your task is to implement clean, efficient HTTP route handlers that provide a complete user management API.

## Your Mission
Implement all API route handlers for user endpoints, connecting HTTP requests to the repository layer while ensuring proper validation, error handling, and RESTful design.

## Context
- Task 4 (User Repository) has been completed
- You have a working repository layer with CRUD operations
- The database and models are already set up
- Focus on creating the HTTP interface layer

## Required Implementations

### 1. Route Handlers (`src/routes/user_routes.rs`)
Implement these five handlers:
- `get_users`: List all users (GET /users)
- `get_user`: Get user by ID (GET /users/:id)
- `create_user`: Create new user (POST /users)
- `update_user`: Update user (PUT /users/:id)
- `delete_user`: Delete user (DELETE /users/:id)

### 2. Module Organization (`src/routes/mod.rs`)
- Export all route handlers
- Add a health check endpoint
- Keep exports clean and organized

### 3. Router Configuration (in `main.rs`)
- Wire up all routes with appropriate HTTP methods
- Inject database pool as state
- Add request tracing middleware
- Configure the server properly

## Technical Requirements
- Use Axum's State extractor for database pool
- Use Path extractor for URL parameters
- Use Json extractor for request bodies
- Validate requests before processing
- Return appropriate HTTP status codes
- Handle errors gracefully

## Handler Patterns

### GET Handlers
```rust
pub async fn handler(
    State(pool): State<PgPool>,
    Path(id): Path<i32>, // if needed
) -> Result<Json<T>>
```

### POST Handler
```rust
pub async fn handler(
    State(pool): State<PgPool>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>)>
```

### PUT Handler
```rust
pub async fn handler(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<User>>
```

### DELETE Handler
```rust
pub async fn handler(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<StatusCode>
```

## Status Code Requirements
- GET success: 200 OK
- POST success: 201 Created
- PUT success: 200 OK
- DELETE success: 204 No Content
- Not found: 404 Not Found
- Validation error: 400 Bad Request
- Server error: 500 Internal Server Error

## Validation Flow
1. Extract request data
2. Call `validate_request(&req)?` for POST/PUT
3. Return early if validation fails
4. Proceed with repository operation

## Error Handling Strategy
- Convert repository Option::None to ApiError::NotFound
- Let ApiError's IntoResponse handle HTTP conversion
- Don't expose internal error details
- Return consistent error format

## Router Setup
```rust
Router::new()
    .route("/health", get(health_check))
    .route("/users", get(get_users).post(create_user))
    .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
    .layer(TraceLayer::new_for_http())
    .with_state(pool)
```

## Expected Deliverables
1. Complete implementation of all five route handlers
2. Module exports properly organized
3. Router configuration in main.rs
4. tower-http added to Cargo.toml
5. All endpoints return correct status codes
6. Validation integrated for POST/PUT

## Quality Checklist
- [ ] All endpoints follow RESTful conventions
- [ ] Proper HTTP methods used
- [ ] Status codes match standards
- [ ] Validation errors return 400
- [ ] Not found errors return 404
- [ ] Request logging enabled
- [ ] No unwrap() or expect() calls
- [ ] Clean error propagation with ?

## Implementation Steps
1. First, add tower-http to dependencies
2. Create the route handlers file
3. Implement each handler systematically
4. Create module exports
5. Update main.rs with router configuration
6. Test each endpoint

## Testing Your Implementation
After implementation, verify:
- POST /users with valid data returns 201
- GET /users returns array (empty or populated)
- GET /users/:id returns user or 404
- PUT /users/:id updates and returns user
- DELETE /users/:id returns 204
- Invalid email returns 400
- Non-existent ID returns 404

## Important Notes
- Repository instances should be created per request
- State(pool) gives you the database connection pool
- validate_request is in models::validation module
- All handlers should be async
- Use Result type for error propagation
- Let Axum handle JSON serialization

## Common Patterns
```rust
// Convert None to NotFound
repo.get_user(id).await?
    .ok_or(ApiError::NotFound)?

// Return with status code
Ok((StatusCode::CREATED, Json(user)))

// Simple status return
Ok(StatusCode::NO_CONTENT)
```

Begin by creating the routes module structure, then implement each handler following the RESTful conventions. Ensure all error cases are handled properly and the API provides a clean, consistent interface.