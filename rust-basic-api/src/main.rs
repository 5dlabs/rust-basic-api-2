#![allow(unexpected_cfgs)]

use std::net::{Ipv4Addr, SocketAddr};

use axum::Server;
use rust_basic_api::{
    application::{build_router, init_tracing},
    config::Config,
    AppResult,
};

#[tokio::main]
async fn main() -> AppResult<()> {
    init_tracing()?;

    let config = Config::from_env()?;
    let has_database_credentials = !config.database_url.is_empty();
    tracing::debug!(has_database_credentials, "Loaded database configuration");

    let router = build_router();
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.server_port));
    tracing::info!(%address, "Listening on address");

    Server::bind(&address)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
