use std::env;

// Test that the binary compiles and can be run
// This tests the main.rs module indirectly

#[test]
fn test_binary_exists() {
    // Verify the binary can be built
    let output = std::process::Command::new("cargo")
        .args(["build", "--bin", "rust-basic-api"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute cargo build");

    assert!(
        output.status.success(),
        "Binary should build successfully"
    );
}

#[test]
fn test_config_can_be_loaded() {
    // Test that config module works correctly
    env::set_var("DATABASE_URL", "postgresql://localhost:5432/testdb");
    env::set_var("SERVER_PORT", "3000");

    let config = rust_basic_api::config::Config::from_env();
    assert!(config.is_ok());

    let config = config.unwrap();
    assert_eq!(config.server_port(), 3000);
    assert_eq!(config.database_url(), "postgresql://localhost:5432/testdb");

    env::remove_var("DATABASE_URL");
    env::remove_var("SERVER_PORT");
}

#[tokio::test]
async fn test_router_can_be_created() {
    // Test that router can be created successfully
    let router = rust_basic_api::routes::router();

    // Verify router is functional by sending a test request
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    let response = router
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_repository_pool_creation() {
    // Test that repository pool can be created
    let url = "postgresql://localhost:5432/testdb";
    let pool = rust_basic_api::repository::create_pool(url, 5);

    assert!(pool.is_ok());
}
