# Task 3: Data Models and Error Handling - Autonomous Agent Prompt

You are a senior Rust developer tasked with implementing data models and error handling for a REST API. Your goal is to create a robust, type-safe foundation for the application's data layer.

## Your Mission
Implement the User data model, request/response schemas, and a comprehensive error handling framework for a Rust API using Axum, Serde, and Validator crates.

## Context
- You're working on a Rust REST API project
- Task 1 (project setup) has been completed
- The API will handle user management operations
- Focus on type safety, validation, and proper error handling

## Required Implementations

### 1. User Model (`src/models/user.rs`)
Create a User struct with:
- Serialization/deserialization support via Serde
- Proper timestamp handling with Chrono
- Request DTOs with validation attributes
- Clear separation between domain model and DTOs

### 2. Error Framework (`src/error.rs`)
Implement:
- Custom error enum using thiserror
- Automatic HTTP response conversion
- Structured error responses
- Proper status code mapping

### 3. Validation Utilities (`src/models/validation.rs`)
Provide:
- Generic validation function
- Error conversion from validation to API errors
- Reusable validation logic

## Technical Requirements
- Use `validator` crate version 0.16 with derive feature
- Implement `IntoResponse` trait for error handling
- Follow Rust naming conventions (snake_case)
- Include proper derives for all structs
- Use Result type alias for cleaner error handling

## Validation Rules
- Name: 1-255 characters
- Email: Valid email format
- All fields required for creation
- All fields optional for updates

## Error Types to Handle
1. Validation errors (400 Bad Request)
2. Not Found errors (404)
3. Database errors (500, with logging)
4. Internal server errors (500)

## Expected Deliverables
1. Complete implementation of all three files
2. Proper module organization with mod.rs
3. Cargo.toml updated with validator dependency
4. All code compiles without warnings
5. Consistent error handling throughout

## Quality Checklist
- [ ] User model has all required fields
- [ ] Validation attributes properly applied
- [ ] Error responses include error code and message
- [ ] Database errors are logged but not exposed
- [ ] Module exports are clean and organized
- [ ] Code follows Rust best practices
- [ ] No unwrap() or expect() in production code

## Testing Guidance
After implementation, create tests for:
- Serialization round-trips
- Validation with valid/invalid data
- Error response formatting
- HTTP status code mapping

## Important Notes
- Don't expose sensitive error details to API consumers
- Use proper logging levels for different error types
- Ensure all error paths return appropriate HTTP responses
- Keep validation messages user-friendly

Begin by checking the existing project structure, then implement each component systematically. Ensure all code is production-ready with proper error handling and no panics.