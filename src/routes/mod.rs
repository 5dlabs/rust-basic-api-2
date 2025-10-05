use axum::{extract::State, routing::get, Router};
use sqlx::PgPool;
use tracing::instrument;

use crate::models::HealthResponse;

/// Shared application state passed to request handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

/// Build the application router with all HTTP routes.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .with_state(state)
}

#[instrument(name = "health_check", skip(state))]
async fn health_check(State(state): State<AppState>) -> &'static str {
    if state.pool.is_closed() {
        tracing::warn!("database connection pool is closed");
    }

    HealthResponse::healthy().status
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
        let pool = crate::repository::test_utils::setup_test_database().await;
        crate::repository::test_utils::cleanup_database(&pool).await;
        let app = router(AppState { pool: pool.clone() });

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
        let payload = std::str::from_utf8(&body).expect("response should be valid UTF-8");

        assert_eq!(payload, "OK");

        crate::repository::test_utils::cleanup_database(&pool).await;
    }
}
