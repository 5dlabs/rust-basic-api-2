use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;

fn default_database_url() -> String {
    let scheme = "postgresql";
    let user = "postgres";
    let password = "postgres";
    let host = "localhost";
    let port = 15432;
    let database = "rust_basic_api_test";

    format!("{scheme}://{user}:{password}@{host}:{port}/{database}")
}

fn database_url_from_env() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        let fallback = default_database_url();
        std::env::set_var("DATABASE_URL", &fallback);
        fallback
    })
}

async fn create_app() -> (Router, rust_basic_api::state::SharedAppState, PgPool) {
    dotenv::from_filename(".env.test").ok();

    let database_url = database_url_from_env();

    let pool = rust_basic_api::repository::create_pool(&database_url)
        .await
        .expect("Failed to create test database pool");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations in integration tests");

    let config = Arc::new(rust_basic_api::config::Config {
        database_url,
        server_port: 3000,
    });

    let state = Arc::new(rust_basic_api::state::AppState::new(config, pool.clone()));

    (
        rust_basic_api::routes::router().with_state(state.clone()),
        state,
        pool,
    )
}

async fn cleanup(pool: &PgPool) {
    sqlx::query("TRUNCATE TABLE users RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("Failed to clean up users table");
}

#[tokio::test]
async fn test_health_endpoint_returns_ok() {
    let (app, _state, pool) = create_app().await;

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

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str, "OK");

    cleanup(&pool).await;
}

#[tokio::test]
async fn test_health_endpoint_with_empty_database_url() {
    let (_router, _state, pool) = create_app().await;

    let config = Arc::new(rust_basic_api::config::Config {
        database_url: String::new(),
        server_port: 3000,
    });

    let state = Arc::new(rust_basic_api::state::AppState::new(config, pool.clone()));
    let app = rust_basic_api::routes::router().with_state(state);

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

    cleanup(&pool).await;
}

#[tokio::test]
async fn test_health_endpoint_with_different_ports() {
    let (_router, _state, pool) = create_app().await;

    let config = Arc::new(rust_basic_api::config::Config {
        database_url: database_url_from_env(),
        server_port: 8080,
    });

    let state = Arc::new(rust_basic_api::state::AppState::new(config, pool.clone()));
    let app = rust_basic_api::routes::router().with_state(state);

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

    cleanup(&pool).await;
}

#[tokio::test]
async fn test_health_endpoint_multiple_requests() {
    let (app, _state, pool) = create_app().await;

    for _ in 0..10 {
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

    cleanup(&pool).await;
}

#[tokio::test]
async fn test_nonexistent_route_returns_404() {
    let (app, _state, pool) = create_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    cleanup(&pool).await;
}

#[tokio::test]
async fn test_health_endpoint_head_method() {
    let (app, _state, pool) = create_app().await;

    let response = app
        .clone()
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

    cleanup(&pool).await;
}

#[tokio::test]
async fn test_health_endpoint_post_method_not_allowed() {
    let (app, _state, pool) = create_app().await;

    let response = app
        .clone()
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

    cleanup(&pool).await;
}

#[tokio::test]
async fn test_router_cloneable() {
    let (router1, _state, pool) = create_app().await;
    let _router2 = router1.clone();
    cleanup(&pool).await;
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
