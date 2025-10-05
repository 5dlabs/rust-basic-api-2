# Acceptance Criteria: Project Setup and Configuration

## Required Deliverables

### 1. Project Structure
- [ ] Rust project created with name `rust-basic-api`
- [ ] Project type is binary (not library)
- [ ] All required directories exist:
  - [ ] `src/models/`
  - [ ] `src/routes/`
  - [ ] `src/repository/`

### 2. Source Files
- [ ] `src/main.rs` exists and contains:
  - [ ] Module declarations for all submodules
  - [ ] Tokio async main function
  - [ ] Tracing initialization
  - [ ] Configuration loading
  - [ ] HTTP server setup
  - [ ] Health check endpoint
- [ ] `src/config.rs` exists and contains:
  - [ ] `Config` struct with database_url and server_port fields
  - [ ] `from_env()` method implementation
  - [ ] Proper error handling for missing environment variables
- [ ] `src/error.rs` exists (can be empty initially)
- [ ] `src/models/mod.rs` exists
- [ ] `src/routes/mod.rs` exists
- [ ] `src/repository/mod.rs` exists

### 3. Configuration Files
- [ ] `Cargo.toml` contains all required dependencies:
  - [ ] axum = "0.6.0" or compatible version
  - [ ] tokio with "full" features
  - [ ] serde with "derive" feature
  - [ ] serde_json
  - [ ] sqlx with PostgreSQL and async runtime features
  - [ ] tracing and tracing-subscriber
  - [ ] dotenv
  - [ ] anyhow
  - [ ] thiserror
- [ ] `env.template` exists with:
  - [ ] DATABASE_URL example
  - [ ] SERVER_PORT example
  - [ ] RUST_LOG example

### 4. Containerization
- [ ] `Dockerfile` exists with:
  - [ ] Multi-stage build (builder and runtime stages)
  - [ ] Rust base image for building
  - [ ] Slim runtime image
  - [ ] Proper COPY commands
  - [ ] EXPOSE 3000 directive

## Functional Tests

### 1. Build Test
```bash
cd rust-basic-api
cargo build
```
**Expected**: Build completes successfully without errors

### 2. Run Test
```bash
cargo run
```
**Expected**: 
- Server starts without panics
- Log message shows "Listening on 0.0.0.0:3000"
- Process continues running

### 3. Health Check Test
```bash
curl http://localhost:3000/health
```
**Expected**: Response body contains "OK"

### 4. Environment Variable Test
```bash
SERVER_PORT=8080 cargo run
```
**Expected**: Server starts on port 8080 instead of 3000

### 5. Docker Build Test
```bash
docker build -t rust-basic-api .
```
**Expected**: Docker image builds successfully

### 6. Container Health Check
```bash
docker run -p 3000:3000 -e DATABASE_URL=your_database_url rust-basic-api
curl http://localhost:3000/health
```
**Expected**: Response "OK" from containerized application

## Non-Functional Requirements

### Code Quality
- [ ] Code follows Rust idioms and best practices
- [ ] Proper use of Result types for error handling
- [ ] No compiler warnings
- [ ] Consistent formatting (cargo fmt)
- [ ] No clippy warnings (cargo clippy)

### Documentation
- [ ] Code includes appropriate comments
- [ ] Module-level documentation where needed
- [ ] README.md with basic project information (optional)

### Performance
- [ ] Server starts within 2 seconds
- [ ] Health endpoint responds within 10ms
- [ ] Memory usage under 50MB at idle

## Definition of Done

1. All required files and directories exist
2. Project compiles without errors or warnings
3. Server runs and responds to health checks
4. Environment variable configuration works
5. Docker image builds successfully
6. All functional tests pass
7. Code meets quality standards