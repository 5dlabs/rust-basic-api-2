#![allow(unexpected_cfgs)]

mod application;
mod config;
mod error;
mod models;
mod repository;
mod routes;

#[cfg(test)]
mod test_support;

use std::future::pending;

use application::{bind_address, build_router, init_tracing, load_config, run_with};
use error::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
    init_tracing();
    let config = load_config()?;
    let router = build_router();
    let address = bind_address(config.server_port);

    run_with(router, address, config, pending()).await
}
