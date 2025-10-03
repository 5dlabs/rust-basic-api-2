mod config;
mod error;
mod models;
mod repository;
mod routes;

use std::net::SocketAddr;

use anyhow::Context;
use axum::{Extension, Server};
use config::Config;
use error::Result;
use models::AppState;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let config = Config::from_env().context("failed to load configuration")?;
    let db_pool = repository::create_pool(&config.database_url)
        .context("failed to initialise database connection pool")?;
    let app = routes::create_router().layer(Extension(AppState::new(db_pool)));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Listening on {addr}");

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("server execution failed")?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
