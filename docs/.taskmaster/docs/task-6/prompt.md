# Task 6: Comprehensive Testing Setup - Autonomous Implementation Prompt

You are a senior Rust developer tasked with implementing a comprehensive testing framework for a Rust API project. Your goal is to set up unit tests, integration tests, test utilities, and continuous integration.

## Your Mission
Implement a complete testing infrastructure that ensures code quality, reliability, and maintainability through automated testing. This includes creating test utilities, configuring test databases, setting up coverage reporting, and implementing CI/CD workflows.

## Context
- Project: Rust-based REST API with PostgreSQL database
- Current State: Core API functionality implemented (Tasks 1-5 completed)
- Dependencies: Actix-web, SQLx, PostgreSQL
- Goal: Establish robust testing practices

## Implementation Requirements

### 1. Test Utilities Module (`src/test_utils.rs`)
Create reusable test helpers:
```rust
#[cfg(test)]
pub mod test_utils {
    use crate::models::User;
    use chrono::Utc;
    
    pub fn create_test_user(id: i32) -> User {
        User {
            id,
            name: format!("Test User {}", id),
            email: format!("test{}@example.com", id),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    // Add more factory functions as needed
}
```

### 2. Test Environment Configuration (`.env.test`)
```
DATABASE_URL=postgresql://postgres:password@localhost:5432/rust_api_test
RUST_LOG=debug
```

### 3. Test Database Setup Script (`scripts/setup_test_db.sh`)
Create an executable script that:
- Manages PostgreSQL Docker container
- Creates test database
- Handles container lifecycle
- Provides health checks

### 4. Coverage Configuration
Add to `Cargo.toml`:
```toml
[dev-dependencies]
tarpaulin = "0.25"
```

### 5. Test Execution Script (`scripts/run_tests.sh`)
Create script that:
- Sets up test database
- Runs tests with coverage
- Generates HTML reports
- Provides clear output

### 6. GitHub Actions Workflow (`.github/workflows/ci.yml`)
Implement CI pipeline with:
- PostgreSQL service container
- Rust toolchain setup
- Dependency caching
- Code quality checks
- Test execution

## Step-by-Step Implementation

1. **Create Test Utilities**
   - Navigate to src directory
   - Create test_utils.rs file
   - Implement factory functions
   - Add module to lib.rs

2. **Configure Test Environment**
   - Create .env.test file
   - Set DATABASE_URL for test database
   - Configure logging levels

3. **Implement Database Setup**
   - Create scripts directory if not exists
   - Write setup_test_db.sh script
   - Make script executable
   - Test script execution

4. **Add Coverage Tooling**
   - Update Cargo.toml dev-dependencies
   - Configure Tarpaulin settings
   - Test coverage generation

5. **Create Test Runner**
   - Write run_tests.sh script
   - Integrate database setup
   - Add coverage reporting
   - Make script executable

6. **Setup CI/CD**
   - Create .github/workflows directory
   - Write ci.yml workflow
   - Configure PostgreSQL service
   - Add caching and optimization

## Testing Checklist
- [ ] Test utilities module created
- [ ] Test database configuration set
- [ ] Database setup script working
- [ ] Coverage tool installed
- [ ] Test runner script functional
- [ ] CI workflow configured
- [ ] All tests passing locally
- [ ] Coverage reports generating
- [ ] CI pipeline running successfully

## Validation Steps
1. Run `./scripts/setup_test_db.sh` to verify database setup
2. Execute `cargo test` to run all tests
3. Run `./scripts/run_tests.sh` for coverage
4. Push to branch to trigger CI
5. Verify coverage report in `./coverage/`

## Expected Outcomes
- Complete test infrastructure in place
- Automated testing on every commit
- Coverage reports available
- CI/CD pipeline operational
- Test utilities ready for use
- Documentation for testing practices

## Error Handling
- Handle Docker container failures gracefully
- Provide clear error messages for setup issues
- Implement retry logic for flaky operations
- Log detailed information for debugging

## Notes
- Ensure all scripts are executable (`chmod +x`)
- Test database should be isolated from development
- Coverage threshold can be configured later
- Consider adding performance benchmarks
- Document testing best practices for team

Remember to test each component thoroughly before moving to the next. The testing infrastructure should be reliable, fast, and easy to maintain.