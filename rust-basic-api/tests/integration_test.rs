use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use rust_basic_api::routes::{create_router, AppState};
use rust_basic_api::{config::Config, repository::test_utils};
use tower::ServiceExt;

#[tokio::test]
async fn test_router_creation_with_real_pool() {
    let pool = test_utils::setup_test_database().await;

    let router = create_router(AppState { pool: pool.clone() });

    let debug_output = format!("{router:?}");
    assert!(!debug_output.is_empty());

    test_utils::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_app_state_clone() {
    let pool = test_utils::setup_test_database().await;

    let state = AppState { pool: pool.clone() };
    let cloned = state.clone();
    cloned
        .pool
        .acquire()
        .await
        .expect("cloned state should provide usable pool");

    test_utils::cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_health_check_endpoint() {
    let pool = test_utils::setup_test_database().await;
    let router = create_router(AppState { pool: pool.clone() });

    let response = router
        .oneshot(
            Request::builder()
                .uri("/health")
                .method("GET")
                .body(Body::empty())
                .expect("failed to build request"),
        )
        .await
        .expect("health check request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    test_utils::cleanup_database(&pool).await;
}

#[test]
fn test_config_integration() {
    std::env::set_var("DATABASE_URL", "postgresql://user:pass@host:5432/db");
    std::env::set_var("SERVER_PORT", "9000");
    std::env::set_var("DATABASE_MAX_CONNECTIONS", "20");
    std::env::set_var("DATABASE_ACQUIRE_TIMEOUT_SECS", "10");

    let config = Config::from_env().expect("Failed to load config");

    assert_eq!(config.database_url, "postgresql://user:pass@host:5432/db");
    assert_eq!(config.server_port, 9000);

    std::env::remove_var("DATABASE_URL");
    std::env::remove_var("SERVER_PORT");
    std::env::remove_var("DATABASE_MAX_CONNECTIONS");
    std::env::remove_var("DATABASE_ACQUIRE_TIMEOUT_SECS");
}
