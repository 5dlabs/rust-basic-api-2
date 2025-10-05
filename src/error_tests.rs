// Integration tests for error handling
use crate::error::{AppError, AppResult};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::Value;

#[test]
fn app_error_from_config_error() {
    let config_error = crate::config::ConfigError::EnvVar {
        var: "TEST_VAR",
        source: std::env::VarError::NotPresent,
    };
    let app_error: AppError = config_error.into();

    match app_error {
        AppError::Config(_) => (),
        _ => panic!("Expected AppError::Config variant"),
    }
}

#[test]
fn app_error_from_sqlx_error() {
    let sqlx_error = sqlx::Error::RowNotFound;
    let app_error: AppError = sqlx_error.into();

    match app_error {
        AppError::Database(_) => (),
        _ => panic!("Expected AppError::Database variant"),
    }
}

#[test]
fn app_error_into_response_returns_internal_server_error() {
    let error = AppError::Config(crate::config::ConfigError::EnvVar {
        var: "TEST",
        source: std::env::VarError::NotPresent,
    });

    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn app_error_response_body_contains_error_message() {
    use hyper::body::to_bytes;

    let error = AppError::Config(crate::config::ConfigError::EnvVar {
        var: "DATABASE_URL",
        source: std::env::VarError::NotPresent,
    });

    let response = error.into_response();
    let body = to_bytes(response.into_body())
        .await
        .expect("body should be readable");

    let json: Value = serde_json::from_slice(&body).expect("should be valid JSON");
    assert!(json["error"].as_str().unwrap().contains("DATABASE_URL"));
}

#[test]
fn app_result_type_alias_works() {
    fn test_function() -> AppResult<String> {
        Ok("success".to_string())
    }

    let result = test_function();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[test]
fn app_result_propagates_errors() {
    fn test_function() -> AppResult<String> {
        let error = AppError::Config(crate::config::ConfigError::EnvVar {
            var: "TEST",
            source: std::env::VarError::NotPresent,
        });
        Err(error)
    }

    let result = test_function();
    assert!(result.is_err());
}

#[test]
fn database_error_converts_properly() {
    let db_errors = vec![
        sqlx::Error::RowNotFound,
        sqlx::Error::PoolTimedOut,
        sqlx::Error::PoolClosed,
    ];

    for db_error in db_errors {
        let app_error: AppError = db_error.into();
        match app_error {
            AppError::Database(_) => (),
            _ => panic!("Expected AppError::Database variant"),
        }
    }
}

#[test]
fn anyhow_error_converts_to_app_error() {
    let anyhow_error = anyhow::anyhow!("test error");
    let app_error: AppError = anyhow_error.into();

    match app_error {
        AppError::Other(_) => (),
        _ => panic!("Expected AppError::Other variant"),
    }
}
