mod config;
mod error;
mod models;
mod repository;
mod routes;

use std::net::SocketAddr;

use anyhow::{Context, Result};
use config::Config;
use repository::Database;
use routes::{router, AppState};
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;

    let config = Config::from_env().map_err(anyhow::Error::new)?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&config.database_url)
        .context("failed to initialize database pool")?;

    let database = Database::new(pool);
    let state = AppState::new(database);

    let app = router().with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    info!(%addr, "Starting HTTP server");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("server error")?;

    Ok(())
}

fn init_tracing() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .map_err(anyhow::Error::new)
}
