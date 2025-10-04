mod config;
mod error;
mod models;
mod repository;
mod routes;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::{Context, Result};
use config::Config;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let config = Config::from_env().context("failed to load configuration")?;

    let app = routes::create_router();

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.server_port);
    let has_database_url = !config.database_url.is_empty();
    tracing::info!(
        address = %addr,
        database_url_configured = has_database_url,
        "HTTP server listening"
    );

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("server encountered an unrecoverable error")?;

    Ok(())
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
