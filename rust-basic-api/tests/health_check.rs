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
async fn health_check_returns_ok() {
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

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"OK");
}
