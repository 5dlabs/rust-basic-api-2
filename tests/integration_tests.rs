use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use hyper::body::to_bytes;
use std::env;
use tower::ServiceExt; // for `oneshot`

// Helper to create router like main.rs does
fn create_app() -> Router {
    Router::new().route("/health", axum::routing::get(health_check))
}

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::test]
async fn test_health_endpoint() {
    // Set required environment variable for the test
    env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");

    let app = create_app();

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

    let body = to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"OK");

    // Clean up
    env::remove_var("DATABASE_URL");
}

#[tokio::test]
async fn test_health_endpoint_method_not_allowed() {
    env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");

    let app = create_app();

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

    // Clean up
    env::remove_var("DATABASE_URL");
}

#[tokio::test]
async fn test_nonexistent_route_404() {
    env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");

    let app = create_app();

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

    // Clean up
    env::remove_var("DATABASE_URL");
}

#[tokio::test]
async fn test_health_endpoint_head_method() {
    env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");

    let app = create_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("HEAD")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // HEAD requests should work for GET endpoints
    assert_eq!(response.status(), StatusCode::OK);

    // Clean up
    env::remove_var("DATABASE_URL");
}

#[tokio::test]
async fn test_health_endpoint_with_query_params() {
    env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");

    let app = create_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health?test=value")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"OK");

    // Clean up
    env::remove_var("DATABASE_URL");
}

#[tokio::test]
async fn test_health_endpoint_headers() {
    env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");

    let app = create_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("User-Agent", "test-client")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"OK");

    // Clean up
    env::remove_var("DATABASE_URL");
}

#[tokio::test]
async fn test_multiple_requests() {
    env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");

    // Test multiple sequential requests to ensure router is properly configured
    for i in 0..5 {
        let app = create_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .header("Request-ID", format!("test-{}", i))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"OK");
    }

    // Clean up
    env::remove_var("DATABASE_URL");
}