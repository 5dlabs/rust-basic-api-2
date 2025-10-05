mod config;
mod error;
mod models;
mod repository;
mod routes;

use std::net::SocketAddr;

use anyhow::Context;
use error::AppResult;
use repository::create_pool;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> AppResult<()> {
    configure_tracing().context("failed to initialise tracing subscriber")?;

    let config =
        config::Config::from_env().context("failed to load configuration from environment")?;
    let pool = create_pool(&config.database_url)
        .context("failed to initialise PostgreSQL connection pool")?;
    let app = routes::router(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    info!("listening on {addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn configure_tracing() -> Result<(), tracing_subscriber::util::TryInitError> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .try_init()
}
