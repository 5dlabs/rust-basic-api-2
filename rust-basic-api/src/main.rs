mod config;
mod error;
mod models;
mod repository;
mod routes;

use std::net::SocketAddr;

use anyhow::Context;
use routes::{create_router, AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = config::Config::from_env()?;
    let db_pool = repository::create_pool(&config.database_url)
        .context("failed to initialize database connection pool")?;

    let router = create_router(AppState { db_pool });

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Listening on {addr}");

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .context("server encountered an unrecoverable error")?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
