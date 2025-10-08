mod config;
mod error;
mod models;
mod repository;
mod routes;

use std::net::{Ipv4Addr, SocketAddr};

use axum::Router;
use config::Config;
use error::AppResult;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> AppResult<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;

    let app: Router = routes::router();

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.server_port));
    tracing::info!(%addr, "Listening on address");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
