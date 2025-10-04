mod config;
mod error;
mod models;
mod repository;
mod routes;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use config::Config;
use error::AppResult;
use routes::create_router;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    init_tracing();

    let config = Config::from_env().context("failed to load configuration")?;

    let db_pool = PgPoolOptions::new()
        .connect_lazy(&config.database_url)
        .context("failed to create database pool")?;

    let state = Arc::new(AppState { db_pool });

    let app = create_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    info!(%addr, "starting http server");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("server error")?;

    Ok(())
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
