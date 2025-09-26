# Task 3: Data Models and Error Handling

## Overview
Implement the foundational data models, request/response schemas, and error handling framework for the Rust API. This task establishes the core data structures and error management patterns that will be used throughout the application.

## Dependencies
- Task 1: Project setup and dependencies must be completed

## Technical Specifications

### 1. User Data Model
Create the primary User model with the following structure:
- **ID**: Integer identifier (i32)
- **Name**: String with validation (1-255 characters)
- **Email**: String with email format validation
- **Created At**: UTC timestamp
- **Updated At**: UTC timestamp

### 2. Request/Response Schemas
Implement separate DTOs for different operations:
- **CreateUserRequest**: For new user creation (name and email required)
- **UpdateUserRequest**: For user updates (name and email optional)
- Both schemas include validation attributes using the `validator` crate

### 3. Error Handling Framework
Establish a comprehensive error handling system with:
- Custom error types using `thiserror`
- Automatic conversion to HTTP responses
- Proper status codes for different error types
- Structured error responses with error codes and messages

## Implementation Guide

### Step 1: Add Dependencies
Update `Cargo.toml` with the validator crate:
```toml
validator = { version = "0.16", features = ["derive"] }
```

### Step 2: Create User Model
1. Create `src/models/user.rs`
2. Define the `User` struct with serde serialization
3. Implement `CreateUserRequest` with validation attributes
4. Implement `UpdateUserRequest` with optional fields

### Step 3: Implement Error Handling
1. Create `src/error.rs`
2. Define `ApiError` enum with common error cases
3. Implement `IntoResponse` for automatic HTTP conversion
4. Create `ErrorResponse` struct for consistent error formatting

### Step 4: Add Validation Utilities
1. Create `src/models/validation.rs`
2. Implement generic validation function
3. Convert validation errors to ApiError

### Step 5: Module Organization
1. Create `src/models/mod.rs`
2. Export public types and functions
3. Ensure proper module structure

## Code Structure

```
src/
├── models/
│   ├── mod.rs         # Module exports
│   ├── user.rs        # User model and DTOs
│   └── validation.rs  # Validation utilities
└── error.rs           # Error handling framework
```

## Key Design Decisions

### Separation of Concerns
- Domain models separate from request/response DTOs
- Validation logic decoupled from business logic
- Error handling centralized for consistency

### Type Safety
- Strong typing with Rust's type system
- Validation at the boundary (request level)
- Compile-time guarantees for data structure

### Error Strategy
- Rich error types with context
- Automatic HTTP response conversion
- Proper logging for debugging

## Testing Requirements

### Unit Tests
1. **Model Serialization**: Verify JSON serialization/deserialization
2. **Validation Logic**: Test valid and invalid inputs
3. **Error Responses**: Confirm proper HTTP status codes

### Integration Points
- Database layer (for future tasks)
- HTTP handlers (for API endpoints)
- Middleware (for request validation)

## Success Criteria
- All models compile without warnings
- Validation rules properly enforced
- Error responses follow REST conventions
- Unit tests pass with 100% coverage of public APIs

## Best Practices
- Use derive macros to reduce boilerplate
- Keep validation rules close to the data structure
- Log errors at appropriate levels (error, warn, info)
- Provide meaningful error messages for debugging

## Common Pitfalls to Avoid
- Don't expose internal error details to clients
- Avoid string-based error handling
- Don't skip validation on optional fields
- Remember to handle database constraint violations

## Related Documentation
- [Serde Documentation](https://serde.rs/)
- [Validator Crate](https://docs.rs/validator/)
- [Thiserror Crate](https://docs.rs/thiserror/)
- [Axum Error Handling](https://docs.rs/axum/latest/axum/error_handling/)