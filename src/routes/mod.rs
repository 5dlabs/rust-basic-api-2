use axum::{extract::State, routing::get, Json, Router};
use sqlx::PgPool;
use tracing::instrument;

use crate::models::HealthResponse;

/// Build the application router with all HTTP routes.
pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .with_state(pool)
}

#[instrument(name = "health_check", skip(pool))]
async fn health_check(State(pool): State<PgPool>) -> Json<HealthResponse> {
    if pool.is_closed() {
        tracing::warn!("database connection pool is closed");
    }

    Json(HealthResponse::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use hyper::body::to_bytes;
    use tower::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn health_check_returns_ok_payload() {
        let pool = crate::repository::create_pool(
            "postgresql://postgres@localhost:5432/rust_basic_api_test",
        )
        .expect("pool should be created lazily");
        let app = router(pool);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/health")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body())
            .await
            .expect("body should be readable");
        let payload: serde_json::Value =
            serde_json::from_slice(&body).expect("response should be valid JSON");

        assert_eq!(payload, serde_json::json!({ "status": "OK" }));
    }
}
