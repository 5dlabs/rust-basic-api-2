use axum::{extract::State, http::StatusCode, routing::get, Router};

use crate::{
    error::{AppError, AppResult},
    models::AppState,
};

/// Construct the application router with all public routes.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .with_state(state)
}

async fn health_check(State(state): State<AppState>) -> AppResult<(StatusCode, &'static str)> {
    if state.pool.is_closed() {
        tracing::warn!("Database pool reported as closed during health check");
        return Err(AppError::service_unavailable("database pool is closed"));
    }

    Ok((StatusCode::OK, "OK"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt; // For oneshot

    #[tokio::test]
    async fn test_create_router() {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgresql://test:test@localhost:5432/test")
            .expect("Failed to create test pool");

        let state = AppState::new(pool);
        let router = create_router(state);

        // Router should be created successfully
        // We'll test actual routing in the next tests
        let _ = router;
    }

    #[tokio::test]
    async fn test_health_check_success() {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgresql://test:test@localhost:5432/test")
            .expect("Failed to create test pool");

        let state = AppState::new(pool);
        let app = create_router(state);

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

        // Response body validation
        // In axum 0.6, we can verify the response was successful
        // without reading the body in tests
    }

    #[tokio::test]
    async fn test_health_check_with_open_pool() {
        // Create a pool that's open
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgresql://test:test@localhost:5432/test")
            .expect("Failed to create test pool");

        let state = AppState::new(pool.clone());

        // Call health_check directly
        let result = health_check(State(state)).await;

        assert!(result.is_ok());
        let (status, body) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "OK");
    }

    #[tokio::test]
    async fn test_router_404_on_unknown_route() {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgresql://test:test@localhost:5432/test")
            .expect("Failed to create test pool");

        let state = AppState::new(pool);
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_health_check_method_get_only() {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgresql://test:test@localhost:5432/test")
            .expect("Failed to create test pool");

        let state = AppState::new(pool);
        let app = create_router(state);

        // POST should not be allowed on /health
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
