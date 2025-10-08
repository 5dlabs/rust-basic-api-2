//! HTTP route definitions and handlers.

use axum::{Router, routing::get};

/// Build the application router with all public routes.
pub fn router() -> Router {
    Router::new().route("/health", get(health_check))
}

/// Health check endpoint used by load balancers and monitoring.
pub async fn health_check() -> &'static str {
    "OK"
}
