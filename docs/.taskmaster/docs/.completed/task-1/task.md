# Task 1: Project Setup and Configuration

## Overview
Initialize the Rust project with Cargo, set up the project structure, and configure dependencies for building a production-ready REST API with Axum framework.

## Technical Requirements

### 1. Project Initialization
- Create a new Rust binary project using Cargo
- Configure the project for async runtime with Tokio
- Set up proper error handling with anyhow and thiserror

### 2. Dependencies Configuration
Update `Cargo.toml` with the following dependencies:

```toml
[dependencies]
axum = "0.6.0"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.6", features = ["runtime-tokio-rustls", "postgres", "chrono", "json"] }
tracing = "0.1"
tracing-subscriber = "0.3"
dotenv = "0.15"
anyhow = "1.0"
thiserror = "1.0"
```

### 3. Project Structure
Create the following directory structure:

```
rust-basic-api/
├── src/
│   ├── main.rs           # Application entry point
│   ├── config.rs         # Configuration management
│   ├── error.rs          # Error types and handling
│   ├── models/           # Data models
│   │   └── mod.rs
│   ├── routes/           # API route handlers
│   │   └── mod.rs
│   └── repository/       # Database interaction layer
│       └── mod.rs
├── Cargo.toml
├── .env.example
├── Dockerfile
└── docker-compose.yml
```

## Implementation Guide

### Step 1: Create New Rust Project
```bash
cargo new rust-basic-api --bin
cd rust-basic-api
```

### Step 2: Configure Dependencies
Replace the contents of `Cargo.toml` with the dependency list above.

### Step 3: Implement Configuration Module
Create `src/config.rs` with environment-based configuration:

```rust
use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv().ok();
        
        let database_url = env::var("DATABASE_URL")?;
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
            
        Ok(Config {
            database_url,
            server_port,
        })
    }
}
```

### Step 4: Implement Main Application
Create `src/main.rs` with basic server setup:

```rust
mod config;
mod error;
mod models;
mod routes;
mod repository;

use config::Config;
use std::net::SocketAddr;
use axum::Router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Load configuration
    let config = Config::from_env()?;
    
    // Build application router
    let app = Router::new()
        .route("/health", axum::routing::get(health_check));
    
    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
```

### Step 5: Create Module Placeholders
Create empty module files:
- `src/error.rs`
- `src/models/mod.rs`
- `src/routes/mod.rs`
- `src/repository/mod.rs`

### Step 6: Docker Configuration
Create `Dockerfile`:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY Cargo.* ./
COPY src ./src
RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /app/target/release/rust-basic-api /app/
EXPOSE 3000
CMD ["./rust-basic-api"]
```

### Step 7: Environment Configuration
Create `.env.example`:

```
DATABASE_URL=postgresql://user:password@your-database-host:5432/your-database
SERVER_PORT=3000
RUST_LOG=info
```

## Dependencies and Prerequisites
- Rust 1.70 or later
- Cargo package manager
- Docker (optional for containerization)
- Access to a PostgreSQL database (via DATABASE_URL environment variable)

## Related Tasks
- Task 2: Database Setup (depends on this task)
- Task 3: API Server Implementation (depends on this task)
- Task 4: User Authentication (depends on this task)