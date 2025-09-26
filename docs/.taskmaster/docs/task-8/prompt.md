# Task 8: API Documentation and Developer Experience - Autonomous Implementation Prompt

You are a senior backend engineer tasked with implementing comprehensive API documentation, structured logging, and developer experience improvements for a Rust API project. Your goal is to make the API self-documenting, observable, and developer-friendly.

## Your Mission
Implement OpenAPI/Swagger documentation, structured JSON logging, and development tooling that enhances the developer experience. Create an environment where developers can easily understand, test, and debug the API.

## Context
- Project: Rust-based REST API with existing endpoints
- Current State: Core API complete (Tasks 1-7)
- Framework: Actix-web with PostgreSQL
- Goal: Professional documentation and observability

## Implementation Requirements

### 1. OpenAPI Dependencies (`Cargo.toml`)
Add required dependencies:
```toml
utoipa = { version = "3.0", features = ["axum"] }
utoipa-swagger-ui = { version = "3.0", features = ["axum"] }
tracing-bunyan-formatter = "0.3"
tracing-log = "0.1"
```

### 2. Documentation Module (`src/docs.rs`)
Create OpenAPI documentation structure with all paths and components properly defined. Include API metadata, tags, and comprehensive endpoint documentation.

### 3. Model Annotations
Update all models with utoipa annotations:
- Add `#[derive(utoipa::ToSchema)]`
- Include example values
- Document field constraints
- Add descriptions where helpful

### 4. Route Documentation
Annotate all route handlers with:
- Path operation details
- Response types and status codes
- Request body schemas
- Query parameter documentation

### 5. Swagger UI Integration (`src/main.rs`)
Mount Swagger UI and serve OpenAPI specification at appropriate endpoints. Ensure the documentation is accessible and auto-updates with code changes.

### 6. Structured Logging (`src/logging.rs`)
Implement JSON-formatted logging with:
- Bunyan format for structured output
- Environment-based log levels
- Request correlation IDs
- Performance metrics

### 7. Development Script (`scripts/dev.sh`)
Create hot-reload development environment that:
- Checks for cargo-watch installation
- Manages PostgreSQL container
- Enables automatic restart on changes
- Provides clear console output

### 8. Environment Configuration (`.env`)
Set up development environment variables for database, logging, and server configuration.

## Step-by-Step Implementation

1. **Update Dependencies**
   - Add utoipa and logging dependencies to Cargo.toml
   - Run cargo build to verify compatibility

2. **Create Documentation Module**
   - Create src/docs.rs file
   - Define OpenApi derive struct
   - List all paths and components
   - Add API metadata and tags

3. **Annotate Models**
   - Update User model with ToSchema
   - Add CreateUserRequest schema
   - Add UpdateUserRequest schema
   - Include ErrorResponse schema
   - Add example values to all fields

4. **Document Endpoints**
   - Add utoipa::path macros to handlers
   - Document all response types
   - Include error responses
   - Add operation descriptions

5. **Integrate Swagger UI**
   - Update main.rs imports
   - Mount SwaggerUi router
   - Serve OpenAPI JSON
   - Add tracing layer

6. **Implement Logging**
   - Create logging.rs module
   - Set up Bunyan formatter
   - Configure environment filter
   - Initialize global subscriber

7. **Create Dev Script**
   - Write scripts/dev.sh
   - Add cargo-watch check
   - Configure Docker Compose
   - Set execute permissions

8. **Configure Environment**
   - Create .env file
   - Set DATABASE_URL
   - Configure RUST_LOG
   - Add SERVER_PORT

## Testing Checklist
- [ ] Swagger UI loads at /swagger-ui
- [ ] OpenAPI JSON valid at /api-docs/openapi.json
- [ ] All endpoints visible in documentation
- [ ] Try it out functionality works
- [ ] Structured logs output JSON format
- [ ] Log levels respond to RUST_LOG
- [ ] Hot reload triggers on file changes
- [ ] Environment variables load correctly
- [ ] API examples are accurate
- [ ] Documentation updates with code

## Validation Steps
1. Start development environment: `./scripts/dev.sh`
2. Access Swagger UI: `http://localhost:3000/swagger-ui`
3. Test endpoint from Swagger UI
4. Verify JSON logs in console
5. Make code change and verify hot-reload
6. Change RUST_LOG and verify log filtering
7. Validate OpenAPI spec with online validator
8. Test all CRUD operations from Swagger

## Expected Outcomes
- Interactive API documentation available
- All endpoints fully documented
- JSON structured logs working
- Development environment streamlined
- Hot-reload improving productivity
- Clear request/response examples
- Comprehensive error documentation

## Quality Standards
- 100% endpoint documentation coverage
- All models have example values
- Logs include request correlation
- No sensitive data in logs
- Documentation auto-generates from code
- Development script is idempotent

## Error Handling
- Document all error response types
- Include error examples in Swagger
- Log errors with appropriate context
- Preserve stack traces in debug mode
- Handle missing environment variables

## Performance Considerations
- Swagger UI disabled in production
- Log level affects performance
- Hot-reload for development only
- Documentation generated at compile-time
- Minimal runtime overhead

## Security Requirements
- No secrets in logs
- Swagger UI behind feature flag
- Sanitize user input in logs
- Document authentication requirements
- Rate limiting documentation

## Best Practices Implementation
1. Keep documentation close to code
2. Use meaningful example data
3. Document edge cases
4. Include curl examples
5. Explain status codes
6. Document rate limits
7. Include authentication examples

## Notes
- Test documentation with actual API calls
- Ensure examples match implementation
- Consider API versioning strategy
- Plan for deprecation notices
- Document breaking changes

Remember to test each feature thoroughly. The documentation should be comprehensive enough that a new developer can understand and use the API without reading the source code. The logging should provide enough context for debugging production issues.