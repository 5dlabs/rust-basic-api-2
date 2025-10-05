//! HTTP route handlers for the Axum application.

use axum::{extract::State, response::IntoResponse, routing::get, Router};

use crate::repository::RepositoryPool;

/// Shared application state propagated to HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    /// Pooled `PostgreSQL` connection handle.
    pub pool: RepositoryPool,
}

/// Build the application router containing all service routes.
pub fn create_router() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}

/// Basic liveness probe endpoint.
async fn health_check(State(_state): State<AppState>) -> impl IntoResponse {
    "OK"
}
