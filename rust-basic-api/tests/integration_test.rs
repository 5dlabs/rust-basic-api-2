use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use rust_basic_api::{repository, routes};
use tower::ServiceExt;

fn create_test_app() -> axum::Router {
    // Setup test database pool
    dotenv::from_filename(".env.test").ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env.test");

    let pool = repository::create_pool(&database_url, 5).expect("Failed to create test pool");

    let state = routes::AppState { pool };
    routes::router(state)
}

#[tokio::test]
async fn test_health_endpoint_returns_200() {
    let app = create_test_app();

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
async fn test_health_endpoint_returns_ok_body() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn test_invalid_route_returns_404() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_health_endpoint_with_get_method() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_endpoint_with_post_method_not_allowed() {
    let app = create_test_app();

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
async fn test_health_endpoint_multiple_requests() {
    let app = create_test_app();

    for _ in 0..5 {
        let response = app
            .clone()
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
