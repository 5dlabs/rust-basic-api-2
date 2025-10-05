//! HTTP route handlers for the Axum application.

use axum::{extract::State, response::IntoResponse, routing::get, Router};

use crate::repository::RepositoryPool;

/// Build the application router containing all service routes.
pub fn create_router(pool: RepositoryPool) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .with_state(pool)
}

/// Basic liveness probe endpoint.
async fn health_check(State(_pool): State<RepositoryPool>) -> impl IntoResponse {
    "OK"
}
