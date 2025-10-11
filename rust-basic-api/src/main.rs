//! Rust Basic API
//!
//! A production-ready REST API built with Axum framework, featuring `PostgreSQL` connectivity,
//! structured logging, and containerization support.

mod config;
mod error;
mod models;
mod repository;
mod routes;

use config::Config;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_basic_api=debug,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration from environment
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded successfully");
    tracing::debug!("Server will listen on port {}", config.server_port);

    // Build application router with all routes
    let app = routes::create_router();

    // Create socket address for the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Starting server on {}", addr);

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}
