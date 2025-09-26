# Task 5: API Route Handlers Implementation

## Overview
Implement the API route handlers for all user endpoints using Axum. This task creates the HTTP interface layer that connects client requests to the business logic, providing a RESTful API for user management operations.

## Dependencies
- Task 4: User repository implementation must be completed

## Technical Specifications

### 1. Route Handlers
Implement five core endpoints:
- **GET /users**: List all users
- **GET /users/:id**: Get specific user by ID
- **POST /users**: Create new user
- **PUT /users/:id**: Update existing user
- **DELETE /users/:id**: Delete user

### 2. Request/Response Handling
- Use Axum extractors for path parameters and JSON bodies
- Validate request data before processing
- Return appropriate HTTP status codes
- Format responses as JSON

### 3. State Management
- Inject database pool via Axum State
- Create repository instances per request
- Ensure thread-safe access to shared resources

### 4. Error Handling
- Convert domain errors to HTTP responses
- Return structured error messages
- Use appropriate status codes for different error types

## Implementation Guide

### Step 1: Add Dependencies
Update `Cargo.toml`:
```toml
tower-http = { version = "0.4", features = ["trace"] }
```

### Step 2: Create Route Handlers
1. Create `src/routes/user_routes.rs`
2. Implement handlers for each endpoint
3. Use extractors for request data
4. Validate input before processing

### Step 3: Wire Up Router
1. Create `src/routes/mod.rs` for exports
2. Update `main.rs` with route definitions
3. Configure router with state injection
4. Add middleware for logging

### Step 4: Status Code Management
- POST returns 201 Created
- DELETE returns 204 No Content
- GET returns 200 OK
- PUT returns 200 OK
- Errors return appropriate 4xx/5xx codes

### Step 5: Validation Integration
- Call `validate_request` before processing
- Return validation errors as 400 Bad Request
- Provide clear error messages

## Code Structure

```
src/
├── routes/
│   ├── mod.rs          # Module exports and health check
│   └── user_routes.rs  # User endpoint handlers
└── main.rs             # Router configuration
```

## Route Handler Patterns

### GET Endpoints
```rust
pub async fn handler(
    State(pool): State<PgPool>,
    Path(params): Path<T>,
) -> Result<Json<Response>>
```

### POST/PUT Endpoints
```rust
pub async fn handler(
    State(pool): State<PgPool>,
    Json(body): Json<Request>,
) -> Result<(StatusCode, Json<Response>)>
```

### DELETE Endpoints
```rust
pub async fn handler(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<StatusCode>
```

## RESTful API Design

### Endpoint Specifications

#### GET /users
- Returns: Array of User objects
- Status: 200 OK
- Empty array if no users exist

#### GET /users/:id
- Returns: Single User object
- Status: 200 OK or 404 Not Found
- Error if user doesn't exist

#### POST /users
- Accepts: CreateUserRequest JSON
- Returns: Created User object
- Status: 201 Created
- Validates input before creation

#### PUT /users/:id
- Accepts: UpdateUserRequest JSON
- Returns: Updated User object
- Status: 200 OK or 404 Not Found
- Partial updates supported

#### DELETE /users/:id
- Returns: No body
- Status: 204 No Content or 404 Not Found
- Idempotent operation

## Middleware Configuration

### Request Tracing
- Use TraceLayer for HTTP request logging
- Log request method, path, and status
- Include timing information
- Configure appropriate log levels

### CORS (Future Enhancement)
- Configure allowed origins
- Set permitted methods
- Handle preflight requests

## Error Response Format
```json
{
  "error": "ERROR_CODE",
  "message": "Human-readable error message"
}
```

## Testing Strategy

### Unit Tests
1. Test each handler in isolation
2. Mock repository responses
3. Verify status codes
4. Check response formatting

### Integration Tests
1. Use test database
2. Test complete request/response cycle
3. Verify database state changes
4. Test error scenarios

### Test Scenarios
- Valid CRUD operations
- Validation errors
- Not found errors
- Database errors
- Concurrent requests

## Performance Considerations

### Connection Pooling
- Repository created per request
- Pool shared via state
- Connections released after use

### Response Caching
- Consider caching GET /users
- Invalidate on mutations
- Use ETags for conditional requests

### Request Validation
- Fail fast on invalid input
- Minimize database calls
- Return early for obvious errors

## Success Criteria
- All five endpoints functional
- Proper HTTP status codes returned
- Request validation working
- Error responses properly formatted
- Integration tests passing
- Request logging enabled

## Best Practices
- Use semantic HTTP methods
- Return appropriate status codes
- Validate input at the boundary
- Keep handlers thin
- Delegate business logic to repository
- Log errors for debugging

## Common Pitfalls to Avoid
- Don't return 200 for errors
- Avoid exposing internal errors
- Don't skip validation
- Remember to handle None cases
- Avoid blocking operations in handlers

## Security Considerations
- Validate all input
- Sanitize error messages
- Rate limiting (future)
- Authentication (future)
- CORS configuration

## API Documentation (OpenAPI)
Future enhancement to add OpenAPI/Swagger documentation:
- Document request/response schemas
- Provide example requests
- List possible error codes
- Include authentication details

## Related Documentation
- [Axum Documentation](https://docs.rs/axum/)
- [Tower HTTP](https://docs.rs/tower-http/)
- [RESTful API Design](https://restfulapi.net/)
- [HTTP Status Codes](https://httpstatuses.com/)