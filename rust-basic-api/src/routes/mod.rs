//! API routes module
//!
//! Defines HTTP route handlers and routing configuration.

use axum::{routing::get, Router};

/// Health check endpoint handler
///
/// Returns a simple "OK" message to indicate the service is running.
pub async fn health_check() -> &'static str {
    "OK"
}

/// Build the application router with all routes
pub fn create_router() -> Router {
    Router::new().route("/health", get(health_check))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"OK");
    }
}
