#![allow(unexpected_cfgs)]

use rust_basic_api::{run, AppResult};

#[tokio::main]
async fn main() -> AppResult<()> {
    run().await
}
