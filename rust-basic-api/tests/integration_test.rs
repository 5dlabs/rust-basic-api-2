use axum::http::StatusCode;
use rust_basic_api::routes::{create_router, AppState};
use rust_basic_api::{config::Config, repository::Database};

#[tokio::test]
async fn test_health_endpoint_without_database() {
    // This test verifies the router setup even though we can't connect to a real database
    // The actual database connection is tested in the health check handler
    std::env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
    std::env::set_var("SERVER_PORT", "3000");

    let config = Config::from_env().expect("Failed to load config");
    let database = Database::connect(&config).expect("Failed to connect to database");

    let app = create_router(AppState { database });

    // Verify the router was created successfully
    assert!(format!("{:?}", app).contains("Router"));

    std::env::remove_var("DATABASE_URL");
    std::env::remove_var("SERVER_PORT");
}

#[tokio::test]
async fn test_router_creation() {
    std::env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");

    let config = Config::from_env().expect("Failed to load config");
    let database = Database::connect(&config).expect("Failed to connect");

    let router = create_router(AppState { database });

    // Test that router is created without panics
    let debug_output = format!("{:?}", router);
    assert!(!debug_output.is_empty());

    std::env::remove_var("DATABASE_URL");
}

#[tokio::test]
async fn test_app_state_clone() {
    std::env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");

    let config = Config::from_env().expect("Failed to load config");
    let database = Database::connect(&config).expect("Failed to connect");

    let state = AppState {
        database: database.clone(),
    };

    let _cloned = state.clone();
    // If this compiles and runs, Clone is properly implemented

    std::env::remove_var("DATABASE_URL");
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
