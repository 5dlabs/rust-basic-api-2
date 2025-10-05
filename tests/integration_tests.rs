// Integration tests for the entire application
use serial_test::serial;
use sqlx::PgPool;
use std::{env, sync::Once};

fn load_test_env() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = dotenv::from_filename(".env.test");
    });
}

fn test_database_url() -> String {
    load_test_env();
    env::var("TEST_DATABASE_URL")
        .or_else(|_| env::var("DATABASE_URL"))
        .expect("DATABASE_URL or TEST_DATABASE_URL must be set for tests")
}

async fn create_test_pool() -> PgPool {
    let pool = rust_basic_api::repository::create_pool(&test_database_url())
        .await
        .expect("failed to create database pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    pool
}

mod config_integration {
    use super::*;
    use rust_basic_api::config::{Config, ConfigError};

    fn reset_env() {
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    #[serial]
    fn config_loads_all_fields_correctly() {
        reset_env();
        env::set_var("DATABASE_URL", "postgresql://user:pass@host:5432/db");
        env::set_var("SERVER_PORT", "9000");

        let config = Config::from_env().expect("should load config");

        assert_eq!(config.database_url, "postgresql://user:pass@host:5432/db");
        assert_eq!(config.server_port, 9000);

        reset_env();
    }

    #[test]
    #[serial]
    fn config_rejects_invalid_port_strings() {
        reset_env();
        env::set_var("DATABASE_URL", "postgresql://host:5432/db");
        env::set_var("SERVER_PORT", "not_a_number");

        let result = Config::from_env();

        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidPort { .. }) => (),
            _ => panic!("Expected InvalidPort error"),
        }

        reset_env();
    }

    #[test]
    #[serial]
    fn config_rejects_port_out_of_range() {
        reset_env();
        env::set_var("DATABASE_URL", "postgresql://host:5432/db");
        env::set_var("SERVER_PORT", "99999"); // Exceeds u16::MAX

        let result = Config::from_env();

        assert!(result.is_err());

        reset_env();
    }

    #[test]
    #[serial]
    fn config_accepts_minimum_port() {
        reset_env();
        env::set_var("DATABASE_URL", "postgresql://host:5432/db");
        env::set_var("SERVER_PORT", "1");

        let config = Config::from_env().expect("should accept port 1");
        assert_eq!(config.server_port, 1);

        reset_env();
    }

    #[test]
    #[serial]
    fn config_accepts_maximum_port() {
        reset_env();
        env::set_var("DATABASE_URL", "postgresql://host:5432/db");
        env::set_var("SERVER_PORT", "65535"); // u16::MAX

        let config = Config::from_env().expect("should accept port 65535");
        assert_eq!(config.server_port, 65535);

        reset_env();
    }

    #[test]
    #[serial]
    fn config_error_displays_helpful_message() {
        let error = ConfigError::EnvVar {
            var: "DATABASE_URL",
            source: env::VarError::NotPresent,
        };

        let message = format!("{error}");
        assert!(message.contains("DATABASE_URL"));
    }

    #[test]
    #[serial]
    fn config_error_invalid_port_displays_value() {
        let error = ConfigError::InvalidPort {
            value: "abc123".to_string(),
            source: "abc123".parse::<u16>().unwrap_err(),
        };

        let message = format!("{error}");
        assert!(message.contains("abc123"));
    }

    #[test]
    #[serial]
    fn config_debug_format_works() {
        reset_env();
        env::set_var("DATABASE_URL", "postgresql://host:5432/db");
        env::set_var("SERVER_PORT", "8080");

        let config = Config::from_env().unwrap();
        let debug_str = format!("{config:?}");

        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("8080"));

        reset_env();
    }

    #[test]
    #[serial]
    fn config_clone_works() {
        reset_env();
        env::set_var("DATABASE_URL", "postgresql://host:5432/db");

        let config1 = Config::from_env().unwrap();
        let config2 = config1.clone();

        assert_eq!(config1.database_url, config2.database_url);
        assert_eq!(config1.server_port, config2.server_port);

        reset_env();
    }

    #[test]
    #[serial]
    fn config_handles_zero_port() {
        reset_env();
        env::set_var("DATABASE_URL", "postgresql://host:5432/db");
        env::set_var("SERVER_PORT", "0");

        let config = Config::from_env().unwrap();
        assert_eq!(config.server_port, 0);

        reset_env();
    }

    #[test]
    #[serial]
    fn config_handles_typical_ports() {
        reset_env();
        let test_ports = vec!["80", "443", "8000", "8080", "3000"];

        for port_str in test_ports {
            env::set_var("DATABASE_URL", "postgresql://host:5432/db");
            env::set_var("SERVER_PORT", port_str);

            let config = Config::from_env().unwrap();
            assert_eq!(config.server_port, port_str.parse::<u16>().unwrap());

            reset_env();
        }
    }
}

mod repository_integration {
    use super::create_test_pool;
    use rust_basic_api::repository::create_pool;

    #[tokio::test]
    async fn create_pool_establishes_active_pool() {
        let pool = create_test_pool().await;
        assert!(!pool.is_closed());
    }

    #[tokio::test]
    async fn create_pool_validates_connection_string_format() {
        let result = create_pool("invalid://connection/string").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_pool_handles_unreachable_database() {
        let result = create_pool("postgresql://postgres@127.0.0.1:1/unreachable").await;
        assert!(result.is_err());
    }
}

mod models_integration {
    use rust_basic_api::models::HealthResponse;

    #[test]
    fn health_response_healthy_constructor() {
        let response = HealthResponse::healthy();
        assert_eq!(response.status, "OK");
    }

    #[test]
    fn health_response_default_is_healthy() {
        let response = HealthResponse::default();
        assert_eq!(response.status, "OK");
    }

    #[test]
    fn health_response_debug_formatting() {
        let response = HealthResponse::healthy();
        let debug_str = format!("{response:?}");
        assert!(debug_str.contains("OK"));
    }

    #[test]
    fn health_response_equality() {
        let r1 = HealthResponse::healthy();
        let r2 = HealthResponse::default();
        assert_eq!(r1, r2);
    }

    #[test]
    fn health_response_serializes_to_json() {
        let response = HealthResponse::healthy();
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("OK"));
        assert!(json.contains("status"));
    }
}

mod routes_integration {
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use hyper::body::to_bytes;
    use rust_basic_api::routes::{router, AppState};
    use tower::ServiceExt;

    async fn app() -> axum::Router {
        let pool = super::create_test_pool().await;
        router(AppState { pool })
    }

    #[tokio::test]
    async fn health_endpoint_returns_200() {
        let app = app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn health_endpoint_returns_ok_text() {
        let app = app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = to_bytes(response.into_body()).await.unwrap();
        let text = std::str::from_utf8(&body).unwrap();
        assert_eq!(text, "OK");
    }

    #[tokio::test]
    async fn unknown_routes_return_404() {
        let app = app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn health_endpoint_only_accepts_get() {
        let app = app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
