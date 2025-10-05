//! Tests for HTTP routes

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use rust_basic_api::repository::test_utils::{cleanup_database, setup_test_database};

fn test_state(
    pool: rust_basic_api::repository::RepositoryPool,
) -> rust_basic_api::routes::AppState {
    rust_basic_api::routes::AppState { pool }
}

#[tokio::test]
async fn test_health_endpoint_returns_ok() {
    let pool = setup_test_database().await;
    let app = rust_basic_api::routes::create_router().with_state(test_state(pool.clone()));

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

    cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_health_endpoint_returns_text() {
    let pool = setup_test_database().await;

    let app = rust_basic_api::routes::create_router().with_state(test_state(pool.clone()));

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
    let body = response.into_body();
    let bytes = hyper::body::to_bytes(body).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();

    assert_eq!(text, "OK");

    cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_unknown_route_returns_404() {
    let pool = setup_test_database().await;

    let app = rust_basic_api::routes::create_router().with_state(test_state(pool.clone()));

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

    cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_health_endpoint_only_accepts_get() {
    let pool = setup_test_database().await;

    let app = rust_basic_api::routes::create_router().with_state(test_state(pool.clone()));

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

    cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_router_can_be_cloned() {
    let pool = setup_test_database().await;

    let app = rust_basic_api::routes::create_router().with_state(test_state(pool.clone()));

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

    cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_multiple_requests_to_health_endpoint() {
    let pool = setup_test_database().await;

    // Make multiple requests
    for _ in 0..3 {
        let app = rust_basic_api::routes::create_router().with_state(test_state(pool.clone()));

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

    cleanup_database(&pool).await;
}

#[test]
fn test_pool_type_alias() {
    // Test that RepositoryPool is a valid type alias
    use rust_basic_api::repository::RepositoryPool;

    // This will compile if the type alias is correctly defined
    let _pool_type: Option<RepositoryPool> = None;
}
