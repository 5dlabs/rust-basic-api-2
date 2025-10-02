use axum::{extract::State, routing::get, Router};

use crate::models::SharedState;

pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .with_state(state)
}

async fn health_check(State(state): State<SharedState>) -> &'static str {
    if state.db_pool.is_closed() {
        tracing::warn!(database_state = "closed", "Connection pool is closed");
    }

    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{AppState, SharedState},
        repository,
    };
    use axum::{body::Body, http::Request, http::StatusCode};
    use hyper::body::to_bytes;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn example_state() -> SharedState {
        let pool = repository::create_pool(
            "postgresql://localhost:5432/example_db",
            repository::DEFAULT_MAX_CONNECTIONS,
            repository::DEFAULT_MIN_CONNECTIONS,
            repository::DEFAULT_ACQUIRE_TIMEOUT_SECS,
            repository::DEFAULT_IDLE_TIMEOUT_SECS,
        )
        .expect("pool should be created lazily");
        Arc::new(AppState::new(pool))
    }

    #[tokio::test]
    async fn health_route_returns_ok() {
        let router = create_router(example_state());
        let request = Request::builder()
            .uri("/health")
            .method("GET")
            .body(Body::empty())
            .expect("request should build");

        let response = router
            .oneshot(request)
            .await
            .expect("health request should be handled");

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body()).await.expect("body to bytes");
        let body_text = String::from_utf8(body.to_vec()).expect("response body should be utf8");

        assert_eq!(body_text, "OK");
    }

    #[tokio::test]
    async fn health_route_warns_when_pool_closed() {
        let state = example_state();
        state.db_pool.close().await;

        let router = create_router(state);
        let request = Request::builder()
            .uri("/health")
            .method("GET")
            .body(Body::empty())
            .expect("request should build");

        let response = router
            .oneshot(request)
            .await
            .expect("health request should be handled");

        assert_eq!(response.status(), StatusCode::OK);
    }
}
