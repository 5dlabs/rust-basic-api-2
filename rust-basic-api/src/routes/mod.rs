//! HTTP route definitions and handlers.

use axum::{routing::get, Router};

/// Build the application router with all public routes.
pub fn router() -> Router {
    Router::new().route("/health", get(health_check))
}

/// Health check endpoint used by load balancers and monitoring.
pub async fn health_check() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_check_returns_ok() {
        assert_eq!(health_check().await, "OK");
    }
}
