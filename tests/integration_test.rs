use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt;

// Helper to create test config
fn create_test_config() -> Arc<rust_basic_api::config::Config> {
    Arc::new(rust_basic_api::config::Config {
        database_url: "postgresql://test:test@localhost/testdb".to_string(),
        server_port: 3000,
    })
}

#[tokio::test]
async fn test_health_endpoint_returns_ok() {
    let config = create_test_config();
    let app = rust_basic_api::routes::router().layer(axum::extract::Extension(config));

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
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str, "OK");
}

#[tokio::test]
async fn test_health_endpoint_with_empty_database_url() {
    let config = Arc::new(rust_basic_api::config::Config {
        database_url: String::new(),
        server_port: 3000,
    });
    let app = rust_basic_api::routes::router().layer(axum::extract::Extension(config));

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
async fn test_health_endpoint_with_different_ports() {
    let config = Arc::new(rust_basic_api::config::Config {
        database_url: "postgresql://localhost/testdb".to_string(),
        server_port: 8080,
    });
    let app = rust_basic_api::routes::router().layer(axum::extract::Extension(config));

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
async fn test_health_endpoint_multiple_requests() {
    let config = create_test_config();

    for _ in 0..10 {
        let app = rust_basic_api::routes::router().layer(axum::extract::Extension(config.clone()));

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

#[tokio::test]
async fn test_nonexistent_route_returns_404() {
    let config = create_test_config();
    let app = rust_basic_api::routes::router().layer(axum::extract::Extension(config));

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
async fn test_health_endpoint_head_method() {
    let config = create_test_config();
    let app = rust_basic_api::routes::router().layer(axum::extract::Extension(config));

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

    // HEAD request to GET endpoint should still work with Axum
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_endpoint_post_method_not_allowed() {
    let config = create_test_config();
    let app = rust_basic_api::routes::router().layer(axum::extract::Extension(config));

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
async fn test_router_cloneable() {
    let router1 = rust_basic_api::routes::router();
    let _router2 = router1.clone();
    // If this compiles, router is cloneable
}

#[tokio::test]
async fn test_config_with_long_database_url() {
    let long_url = format!(
        "postgresql://user:pass@host.example.com:5432/database?{}",
        "x=y&".repeat(100)
    );
    let config = Arc::new(rust_basic_api::config::Config {
        database_url: long_url.clone(),
        server_port: 3000,
    });

    assert_eq!(config.database_url, long_url);
}

#[tokio::test]
async fn test_config_with_special_characters_in_database_url() {
    let special_url = "postgresql://user%40:p%40ss@host:5432/db?key=val%20ue";
    let config = Arc::new(rust_basic_api::config::Config {
        database_url: special_url.to_string(),
        server_port: 3000,
    });

    assert_eq!(config.database_url, special_url);
}
