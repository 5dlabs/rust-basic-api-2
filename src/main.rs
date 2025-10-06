mod config;
mod error;
mod models;
mod repository;
mod routes;

use std::{net::SocketAddr, sync::Arc};

use axum::{extract::Extension, Router};
use config::Config;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = Arc::new(Config::from_env()?);
    tracing::debug!(
        database_url_length = config.database_url.len(),
        "configuration loaded"
    );

    let router = build_router(config.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("listening on {addr}");

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if let Err(error) = tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .try_init()
    {
        tracing::warn!(%error, "failed to initialize global tracing subscriber");
    }
}

fn build_router(config: Arc<Config>) -> Router {
    routes::router().layer(Extension(config))
}

async fn shutdown_signal() {
    match signal::ctrl_c().await {
        Ok(()) => tracing::info!("shutdown signal received"),
        Err(error) => tracing::error!(%error, "failed to listen for shutdown signal"),
    }
}
