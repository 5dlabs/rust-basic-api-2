# Task 5: API Route Handlers Implementation - Acceptance Criteria

## Definition of Done
This task is considered complete when all the following criteria are met:

## Core Requirements

### 1. Route Handler Implementation
- [ ] `src/routes/user_routes.rs` file exists and compiles
- [ ] All five route handlers implemented (get_users, get_user, create_user, update_user, delete_user)
- [ ] Each handler properly extracts request data
- [ ] Handlers create repository instances correctly
- [ ] All handlers are async functions

### 2. Request Processing
- [ ] State extractor used for database pool
- [ ] Path extractor used for ID parameters
- [ ] Json extractor used for request bodies
- [ ] Validation called before processing POST/PUT
- [ ] Proper error propagation with ? operator

### 3. Response Formatting
- [ ] All successful responses return JSON
- [ ] Error responses use consistent format
- [ ] Appropriate HTTP status codes returned
- [ ] Empty responses for DELETE (204)

### 4. Module Organization
- [ ] `src/routes/mod.rs` exports all handlers
- [ ] Health check endpoint included
- [ ] Clean module structure

### 5. Router Configuration
- [ ] Main.rs updated with all routes
- [ ] Routes mapped to correct HTTP methods
- [ ] Database pool injected as state
- [ ] TraceLayer middleware configured

### 6. Dependencies
- [ ] tower-http added to Cargo.toml with trace feature
- [ ] Project compiles with new dependency

## Functional Test Cases

### Test Case 1: Create User (POST /users)
```http
POST /users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}

Expected Response:
Status: 201 Created
{
  "id": 1,
  "name": "John Doe",
  "email": "john@example.com",
  "created_at": "...",
  "updated_at": "..."
}
```

### Test Case 2: Get All Users (GET /users)
```http
GET /users

Expected Response:
Status: 200 OK
[
  {
    "id": 1,
    "name": "John Doe",
    "email": "john@example.com",
    "created_at": "...",
    "updated_at": "..."
  }
]
```

### Test Case 3: Get User by ID (GET /users/:id)
```http
GET /users/1

Expected Response:
Status: 200 OK
{
  "id": 1,
  "name": "John Doe",
  "email": "john@example.com",
  "created_at": "...",
  "updated_at": "..."
}
```

### Test Case 4: Update User (PUT /users/:id)
```http
PUT /users/1
Content-Type: application/json

{
  "name": "Jane Doe"
}

Expected Response:
Status: 200 OK
{
  "id": 1,
  "name": "Jane Doe",
  "email": "john@example.com",
  "created_at": "...",
  "updated_at": "..." // newer timestamp
}
```

### Test Case 5: Delete User (DELETE /users/:id)
```http
DELETE /users/1

Expected Response:
Status: 204 No Content
(no body)
```

## Error Handling Test Cases

### Validation Error
```http
POST /users
Content-Type: application/json

{
  "name": "",
  "email": "invalid-email"
}

Expected Response:
Status: 400 Bad Request
{
  "error": "VALIDATION_ERROR",
  "message": "..."
}
```

### Not Found Error
```http
GET /users/9999

Expected Response:
Status: 404 Not Found
{
  "error": "NOT_FOUND",
  "message": "Resource not found"
}
```

### Missing Required Fields
```http
POST /users
Content-Type: application/json

{
  "name": "John"
}

Expected Response:
Status: 400 Bad Request or 422 Unprocessable Entity
```

## Non-Functional Requirements

### Performance
- [ ] Response time < 100ms for single operations
- [ ] Can handle 100 concurrent requests
- [ ] No memory leaks in handlers
- [ ] Efficient database pool usage

### Security
- [ ] Input validation prevents injection
- [ ] Error messages don't leak sensitive info
- [ ] Headers properly set for JSON responses
- [ ] No credentials in responses

### Code Quality
- [ ] No compiler warnings
- [ ] No use of unwrap() or expect()
- [ ] Consistent error handling
- [ ] Clear function signatures
- [ ] Follows Rust conventions

### Logging
- [ ] Request method and path logged
- [ ] Response status logged
- [ ] Request duration tracked
- [ ] Errors logged with context

## HTTP Standards Compliance

### RESTful Design
- [ ] GET is idempotent and safe
- [ ] POST creates resources
- [ ] PUT updates resources
- [ ] DELETE removes resources
- [ ] Proper use of status codes

### Status Code Requirements
- [ ] 200 OK for successful GET/PUT
- [ ] 201 Created for successful POST
- [ ] 204 No Content for successful DELETE
- [ ] 400 Bad Request for validation errors
- [ ] 404 Not Found for missing resources
- [ ] 500 Internal Server Error for server issues

### Content Type Handling
- [ ] Accepts application/json for POST/PUT
- [ ] Returns application/json for all responses
- [ ] Proper Content-Type headers set

## Integration Requirements
- [ ] Works with SqlxUserRepository from Task 4
- [ ] Uses User model from Task 3
- [ ] Uses ApiError from Task 3
- [ ] Uses validation from Task 3
- [ ] Compatible with database from Task 2

## Router Configuration
- [ ] All routes properly mapped
- [ ] Middleware chain correct
- [ ] State injection working
- [ ] Server binds to correct address
- [ ] Graceful shutdown supported

## Testing Requirements
- [ ] All endpoints manually testable
- [ ] curl/Postman collection works
- [ ] Integration tests can be written
- [ ] Error cases properly tested
- [ ] Concurrent requests handled

## API Consistency
- [ ] Consistent URL patterns (/users, /users/:id)
- [ ] Consistent error format across endpoints
- [ ] Consistent JSON field naming
- [ ] Predictable behavior for all operations

## Documentation
- [ ] Each handler has clear purpose
- [ ] Complex logic commented
- [ ] API behavior documented
- [ ] Example requests/responses provided

## Middleware Functionality
- [ ] Request logging active
- [ ] Timing information captured
- [ ] Error responses logged
- [ ] Tracing context propagated

## Verification Steps
1. Start server with `cargo run`
2. Test health endpoint: `curl localhost:3000/health`
3. Create user: `curl -X POST -H "Content-Type: application/json" -d '{"name":"Test","email":"test@example.com"}' localhost:3000/users`
4. List users: `curl localhost:3000/users`
5. Get specific user: `curl localhost:3000/users/1`
6. Update user: `curl -X PUT -H "Content-Type: application/json" -d '{"name":"Updated"}' localhost:3000/users/1`
7. Delete user: `curl -X DELETE localhost:3000/users/1`
8. Verify validation: Send invalid email format
9. Verify 404: Request non-existent user

## Performance Benchmarks
- [ ] POST /users: < 50ms average
- [ ] GET /users: < 30ms average
- [ ] GET /users/:id: < 20ms average
- [ ] PUT /users/:id: < 40ms average
- [ ] DELETE /users/:id: < 30ms average

## Rollback Plan
If issues are found:
1. Check route configuration in main.rs
2. Verify handler signatures match Axum requirements
3. Review extractor usage
4. Check repository integration
5. Rollback to previous working version if critical

## Sign-off Checklist
- [ ] Developer: All endpoints tested manually
- [ ] API: RESTful standards followed
- [ ] Performance: Response times acceptable
- [ ] Security: Input validation working
- [ ] Integration: Works with existing code