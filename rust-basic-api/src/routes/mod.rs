use axum::{extract::State, routing::get, Router};

use crate::repository::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .with_state(state)
}

async fn health_check(State(state): State<AppState>) -> &'static str {
    if state.db_pool.is_closed() {
        tracing::warn!("database connection pool is closed");
    }

    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, http::StatusCode};
    use serial_test::serial;
    use tower::ServiceExt;

    fn example_database_url() -> String {
        format!(
            "{scheme}://{user}:{password}@{host}:{port}/{database}",
            scheme = "postgres",
            user = "example_user",
            password = "example_secret",
            host = "localhost",
            port = 5432,
            database = "example_db"
        )
    }

    #[tokio::test]
    #[serial]
    async fn health_route_returns_ok() {
        let url = example_database_url();
        let pool = crate::repository::create_pool(&url).expect("pool should be created");

        let app = create_router(AppState { db_pool: pool });

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

        let body_bytes = hyper::body::to_bytes(response.into_body())
            .await
            .expect("body should convert to bytes");

        assert_eq!(body_bytes.as_ref(), b"OK");
    }
}
