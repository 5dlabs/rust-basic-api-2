# Autonomous Agent Prompt: Project Setup and Configuration

## Task Objective
You are tasked with initializing a new Rust REST API project using the Axum framework. This project will serve as the foundation for a production-ready API with database connectivity, proper error handling, and containerization support.

## Requirements

### 1. Project Creation
- Initialize a new Rust binary project named `rust-basic-api`
- Use Cargo as the build system
- Configure for async runtime with Tokio

### 2. Dependency Setup
Configure the following dependencies in `Cargo.toml`:
- `axum` for the web framework
- `tokio` with full features for async runtime
- `serde` and `serde_json` for serialization
- `sqlx` for database connectivity with PostgreSQL support
- `tracing` and `tracing-subscriber` for logging
- `dotenv` for environment variable management
- `anyhow` and `thiserror` for error handling

### 3. Project Structure
Create a modular project structure with:
- Configuration management module (`config.rs`)
- Error handling module (`error.rs`)
- Models directory for data structures
- Routes directory for API endpoints
- Repository directory for database interactions

### 4. Core Implementation
Implement:
- Configuration loading from environment variables
- Basic HTTP server with Axum
- Health check endpoint at `/health`
- Structured logging with tracing
- Graceful error handling

### 5. Containerization
Create:
- Multi-stage Dockerfile for optimized builds
- Environment variable configuration

## Execution Steps

1. **Initialize Project**
   ```bash
   cargo new rust-basic-api --bin
   cd rust-basic-api
   ```

2. **Update Dependencies**
   - Replace `Cargo.toml` dependencies section with the required packages

3. **Create Project Structure**
   - Create all necessary directories and module files
   - Implement module declarations

4. **Implement Configuration**
   - Create `Config` struct with database URL and server port
   - Implement `from_env()` method for loading from environment

5. **Implement Main Application**
   - Set up tracing subscriber for logging
   - Load configuration from environment
   - Create Axum router with health endpoint
   - Start HTTP server on configured port

6. **Create Docker Configuration**
   - Write multi-stage Dockerfile

7. **Create Environment Template**
   - Create `env.template` with all required variables

## Validation

Verify the implementation by:
1. Running `cargo build` to ensure compilation
2. Starting the server with `cargo run`
3. Testing health endpoint: `curl http://localhost:3000/health`
4. Building Docker image: `docker build -t rust-basic-api .`
5. Running with Docker: `docker run -p 3000:3000 -e DATABASE_URL=your_database_url rust-basic-api`

## Expected Outcome

A fully initialized Rust project with:
- Working HTTP server on port 3000
- Health check endpoint returning "OK"
- Proper logging to console
- Docker containerization support
- Database connectivity via DATABASE_URL environment variable
- Modular code structure for future expansion

## Notes
- Ensure all module files are created even if empty initially
- Use proper Rust idioms and error handling patterns
- Follow Rust naming conventions (snake_case for files and functions)
- Include proper error propagation with `?` operator
- Use structured logging with tracing macros