#![allow(unexpected_cfgs)]

mod application;
mod config;
mod error;
mod models;
mod repository;
mod routes;

#[cfg(test)]
mod test_support;

use error::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
    application::run().await
}
