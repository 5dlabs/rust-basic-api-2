use std::sync::Arc;

use axum::{Router, extract::Extension, routing::get};
use tracing::{instrument, trace};

use crate::{config::Config, error::AppResult};

pub type AppState = Arc<Config>;

pub fn router() -> Router {
    Router::new().route("/health", get(health_check))
}

#[instrument(name = "routes.health", skip(config))]
async fn health_check(Extension(config): Extension<AppState>) -> AppResult<&'static str> {
    trace!(
        has_database_url = !config.database_url.is_empty(),
        "health check invoked"
    );
    Ok("OK")
}
