//! Tests for error types

use anyhow::anyhow;

#[test]
fn test_config_error_missing() {
    use rust_basic_api::error::ConfigError;

    let error = ConfigError::Missing("DATABASE_URL");
    let error_msg = error.to_string();

    assert!(error_msg.contains("DATABASE_URL"));
    assert!(error_msg.contains("missing"));
}

#[test]
fn test_config_error_invalid() {
    use rust_basic_api::error::ConfigError;

    let source = anyhow!("parse error");
    let error = ConfigError::invalid("SERVER_PORT", source);
    let error_msg = error.to_string();

    assert!(error_msg.contains("SERVER_PORT"));
    assert!(error_msg.contains("invalid"));
}

#[test]
fn test_app_error_from_config_error() {
    use rust_basic_api::error::{AppError, ConfigError};

    let config_error = ConfigError::Missing("TEST_VAR");
    let app_error: AppError = config_error.into();

    let error_msg = format!("{app_error}");
    assert!(error_msg.contains("TEST_VAR"));
}

#[test]
fn test_app_error_from_sqlx_error() {
    use rust_basic_api::error::AppError;

    let sqlx_error = sqlx::Error::RowNotFound;
    let app_error: AppError = sqlx_error.into();

    let error_msg = format!("{app_error}");
    assert!(!error_msg.is_empty());
}

#[test]
fn test_app_error_from_anyhow() {
    use rust_basic_api::error::AppError;

    let anyhow_error = anyhow!("runtime error");
    let app_error: AppError = anyhow_error.into();

    let error_msg = format!("{app_error}");
    assert!(error_msg.contains("runtime error"));
}

#[test]
fn test_app_result_type() {
    use rust_basic_api::error::{AppError, AppResult};

    // Test Ok case
    let ok_result: AppResult<i32> = Ok(42);
    assert!(ok_result.is_ok());
    if let Ok(value) = ok_result {
        assert_eq!(value, 42);
    }

    // Test Err case
    let err_result: AppResult<i32> = Err(AppError::Runtime(anyhow!("test error")));
    assert!(err_result.is_err());
}

#[test]
fn test_config_error_debug() {
    use rust_basic_api::error::ConfigError;

    let error = ConfigError::Missing("TEST");
    let debug_str = format!("{error:?}");

    assert!(debug_str.contains("Missing"));
    assert!(debug_str.contains("TEST"));
}

#[test]
fn test_app_error_debug() {
    use rust_basic_api::error::AppError;

    let error = AppError::Runtime(anyhow!("test"));
    let debug_str = format!("{error:?}");

    assert!(debug_str.contains("Runtime"));
}

#[test]
fn test_error_conversion_chain() {
    use rust_basic_api::error::{AppResult, ConfigError};

    // Create a function that returns AppResult
    fn test_function() -> AppResult<()> {
        Err(ConfigError::Missing("TEST").into())
    }

    let result = test_function();
    assert!(result.is_err());
}

#[test]
fn test_config_error_invalid_helper() {
    use rust_basic_api::error::ConfigError;

    let error = ConfigError::invalid("PORT", anyhow!("not a number"));
    let error_str = error.to_string();

    assert!(error_str.contains("PORT"));
    assert!(error_str.contains("invalid"));
}

#[test]
fn test_error_sources() {
    use rust_basic_api::error::ConfigError;

    let source_error = anyhow!("source error");
    let config_error = ConfigError::invalid("FIELD", source_error);

    let error_str = config_error.to_string();
    assert!(error_str.contains("FIELD"));
    assert!(error_str.contains("source error"));
}

#[test]
fn test_all_error_variants() {
    use rust_basic_api::error::{AppError, ConfigError};

    // Config error variant
    let config_err = AppError::Config(ConfigError::Missing("TEST"));
    assert!(format!("{config_err}").contains("TEST"));

    // Database error variant
    let db_err = AppError::Database(sqlx::Error::RowNotFound);
    assert!(!format!("{db_err}").is_empty());

    // Runtime error variant
    let runtime_err = AppError::Runtime(anyhow!("runtime"));
    assert!(format!("{runtime_err}").contains("runtime"));
}
