//! Tests for HTTP routes

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint_returns_ok() {
    // Create a lazy pool for testing
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgresql://test:test@localhost:5432/test")
        .expect("Failed to create test pool");

    // Import the router creation function
    let app = rust_basic_api::routes::create_router(pool);

    // Test the health endpoint
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
}

#[tokio::test]
async fn test_health_endpoint_returns_text() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgresql://test:test@localhost:5432/test")
        .expect("Failed to create test pool");

    let app = rust_basic_api::routes::create_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Extract body
    use axum::body::HttpBody;
    let body = response.into_body();
    let bytes = hyper::body::to_bytes(body).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();

    assert_eq!(text, "OK");
}

#[tokio::test]
async fn test_unknown_route_returns_404() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgresql://test:test@localhost:5432/test")
        .expect("Failed to create test pool");

    let app = rust_basic_api::routes::create_router(pool);

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
async fn test_health_endpoint_only_accepts_get() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgresql://test:test@localhost:5432/test")
        .expect("Failed to create test pool");

    let app = rust_basic_api::routes::create_router(pool);

    // POST should not be allowed
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

#[tokio::test]
async fn test_router_can_be_cloned() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgresql://test:test@localhost:5432/test")
        .expect("Failed to create test pool");

    let app = rust_basic_api::routes::create_router(pool.clone());

    // Verify we can clone the pool
    let _cloned_pool = pool.clone();

    // Use the router
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
}

#[tokio::test]
async fn test_multiple_requests_to_health_endpoint() {
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_lazy("postgresql://test:test@localhost:5432/test")
        .expect("Failed to create test pool");

    // Make multiple requests
    for _ in 0..3 {
        let app = rust_basic_api::routes::create_router(pool.clone());

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
    }
}

#[test]
fn test_pool_type_alias() {
    // Test that RepositoryPool is a valid type alias
    use rust_basic_api::repository::RepositoryPool;

    // This will compile if the type alias is correctly defined
    let _pool_type: Option<RepositoryPool> = None;
}
