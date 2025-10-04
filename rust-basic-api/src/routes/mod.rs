use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};

/// Construct the router for the HTTP service.
pub fn create_router() -> Router {
    Router::new().route("/health", get(health_check))
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use hyper::body::to_bytes;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_check_returns_ok() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body())
            .await
            .expect("body should decode");
        assert_eq!(body_bytes.as_ref(), b"OK");
    }
}
