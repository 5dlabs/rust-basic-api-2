mod config;
mod error;
mod models;
mod repository;
mod routes;

use anyhow::Context;
use config::Config;
use models::AppState;
use repository::create_pool;
use std::net::SocketAddr;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = Config::from_env().context("failed to load configuration")?;
    let db_pool = create_pool(&config.database_url)
        .context("failed to initialize database connection pool")?;

    let state = AppState::new(db_pool);
    let router = routes::create_router(state);

    let address = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!(%address, "HTTP server listening");

    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(%error, "failed to listen for shutdown signal");
        return;
    }

    tracing::info!("Shutdown signal received, commencing graceful shutdown");
}
