use std::sync::Arc;

use axum::{routing::get, Extension, Router};

use crate::AppState;

pub type SharedState = Arc<AppState>;

/// Builds the application router with all registered routes.
pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .layer(Extension(state))
}

async fn health_check(Extension(_state): Extension<SharedState>) -> &'static str {
    "OK"
}
