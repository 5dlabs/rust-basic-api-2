use anyhow::Context;
use axum::Router;
use rust_basic_api::{config::Config, repository, routes};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = Config::from_env().context("failed to load configuration")?;

    let _pool = repository::create_pool(config.database_url(), config.database_max_connections())
        .context("failed to configure database connection pool")?;

    let app: Router = routes::router();

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port()));
    tracing::info!(%addr, "starting HTTP server");

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
