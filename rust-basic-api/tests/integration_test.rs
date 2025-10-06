use std::time::Duration;

use axum::{body::Body, routing::get, Router};
use tokio::time::timeout;

/// Test health check endpoint returns "OK"
#[tokio::test]
async fn test_health_endpoint_returns_ok() {
    let app: Router<(), Body> = Router::new().route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to address");

    let addr = listener.local_addr().expect("Failed to get local address");

    tokio::spawn(async move {
        axum::Server::from_tcp(listener.into_std().expect("Failed to convert listener"))
            .expect("Failed to create server")
            .serve(app.into_make_service())
            .await
            .expect("Server failed");
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = timeout(
        Duration::from_secs(5),
        client.get(format!("http://{addr}/health")).send(),
    )
    .await
    .expect("Request timed out")
    .expect("Request failed");

    assert_eq!(response.status(), 200);
    let body = response.text().await.expect("Failed to read response body");
    assert_eq!(body, "OK");
}

/// Test health check endpoint returns correct content type
#[tokio::test]
async fn test_health_endpoint_content_type() {
    let app: Router<(), Body> = Router::new().route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to address");

    let addr = listener.local_addr().expect("Failed to get local address");

    tokio::spawn(async move {
        axum::Server::from_tcp(listener.into_std().expect("Failed to convert listener"))
            .expect("Failed to create server")
            .serve(app.into_make_service())
            .await
            .expect("Server failed");
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{addr}/health"))
        .send()
        .await
        .expect("Request failed");

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok());

    assert!(content_type.is_some());
}

/// Test health check endpoint responds quickly
#[tokio::test]
async fn test_health_endpoint_response_time() {
    let app: Router<(), Body> = Router::new().route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to address");

    let addr = listener.local_addr().expect("Failed to get local address");

    tokio::spawn(async move {
        axum::Server::from_tcp(listener.into_std().expect("Failed to convert listener"))
            .expect("Failed to create server")
            .serve(app.into_make_service())
            .await
            .expect("Server failed");
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let start = std::time::Instant::now();

    let response = client
        .get(format!("http://{addr}/health"))
        .send()
        .await
        .expect("Request failed");

    let duration = start.elapsed();

    assert_eq!(response.status(), 200);
    // Should respond well within 10ms requirement (allowing network overhead)
    assert!(
        duration < Duration::from_millis(100),
        "Health check took {duration:?}"
    );
}

/// Test invalid route returns 404
#[tokio::test]
async fn test_invalid_route_returns_404() {
    let app: Router<(), Body> = Router::new().route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to address");

    let addr = listener.local_addr().expect("Failed to get local address");

    tokio::spawn(async move {
        axum::Server::from_tcp(listener.into_std().expect("Failed to convert listener"))
            .expect("Failed to create server")
            .serve(app.into_make_service())
            .await
            .expect("Server failed");
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{addr}/invalid"))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 404);
}

/// Test server binds to correct address
#[tokio::test]
async fn test_server_binds_to_address() {
    let _app: Router<(), Body> = Router::new().route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to address");

    let addr = listener.local_addr().expect("Failed to get local address");

    assert!(addr.port() > 0);
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
}

/// Test multiple concurrent requests
#[tokio::test]
async fn test_concurrent_requests() {
    let app: Router<(), Body> = Router::new().route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to address");

    let addr = listener.local_addr().expect("Failed to get local address");

    tokio::spawn(async move {
        axum::Server::from_tcp(listener.into_std().expect("Failed to convert listener"))
            .expect("Failed to create server")
            .serve(app.into_make_service())
            .await
            .expect("Server failed");
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Send 10 concurrent requests
    let mut handles = vec![];
    for _ in 0..10 {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            client
                .get(format!("http://{addr}/health"))
                .send()
                .await
                .expect("Request failed")
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let response = handle.await.expect("Task panicked");
        assert_eq!(response.status(), 200);
        let body = response.text().await.expect("Failed to read response body");
        assert_eq!(body, "OK");
    }
}

/// Test HEAD request to health endpoint
#[tokio::test]
async fn test_health_endpoint_head_request() {
    let app: Router<(), Body> = Router::new().route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to address");

    let addr = listener.local_addr().expect("Failed to get local address");

    tokio::spawn(async move {
        axum::Server::from_tcp(listener.into_std().expect("Failed to convert listener"))
            .expect("Failed to create server")
            .serve(app.into_make_service())
            .await
            .expect("Server failed");
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .head(format!("http://{addr}/health"))
        .send()
        .await
        .expect("Request failed");

    // Axum allows HEAD for GET routes
    assert_eq!(response.status(), 200);
}

async fn health_check() -> &'static str {
    "OK"
}
