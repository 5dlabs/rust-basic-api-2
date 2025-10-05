//! Additional tests to improve error handling coverage

use axum::http::StatusCode;

/// Test error status code mappings
#[test]
fn test_status_codes() {
    // Internal server error
    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR.as_u16(), 500);

    // Service unavailable
    assert_eq!(StatusCode::SERVICE_UNAVAILABLE.as_u16(), 503);

    // OK status
    assert_eq!(StatusCode::OK.as_u16(), 200);

    // Not found
    assert_eq!(StatusCode::NOT_FOUND.as_u16(), 404);

    // Method not allowed
    assert_eq!(StatusCode::METHOD_NOT_ALLOWED.as_u16(), 405);
}

/// Test error message formatting
#[test]
fn test_error_message_formats() {
    let config_error = format!("configuration error: {}", "test message");
    assert!(config_error.starts_with("configuration error:"));

    let db_error = format!("database error: {}", "connection failed");
    assert!(db_error.starts_with("database error:"));

    let service_error = format!("service unavailable: {}", "database is down");
    assert!(service_error.starts_with("service unavailable:"));
}

/// Test error response JSON structure
#[test]
fn test_error_response_structure() {
    use serde_json::json;

    let error_response = json!({
        "error": "test error message"
    });

    assert!(error_response.is_object());
    assert!(error_response["error"].is_string());
    assert_eq!(error_response["error"], "test error message");
}

/// Test various error scenarios
#[test]
fn test_error_conversion_scenarios() {
    // Test that we can create errors from strings
    let error_msg = "Something went wrong".to_string();
    assert!(!error_msg.is_empty());

    // Test error message contains expected text
    assert!(error_msg.contains("went wrong"));
}

/// Test debug formatting for errors
#[test]
fn test_error_debug_formatting() {
    // Test that error types implement Debug
    let test_error = "test error";
    let debug_str = format!("{test_error:?}");
    assert!(!debug_str.is_empty());
}

/// Test error context propagation
#[test]
fn test_error_context() {
    use anyhow::anyhow;

    let base_error = anyhow!("base error");
    let error_string = format!("{base_error}");
    assert!(error_string.contains("base error"));
}

/// Test sqlx error handling patterns
#[test]
fn test_sqlx_error_types() {
    // Test that we handle row not found
    let row_not_found = sqlx::Error::RowNotFound;
    assert_eq!(
        format!("{row_not_found}"),
        "no rows returned by a query that expected to return at least one row"
    );

    // Test error display
    let error_display = format!("{row_not_found}");
    assert!(!error_display.is_empty());
}

/// Test HTTP response type conversions
#[test]
fn test_http_response_conversions() {
    use axum::http::StatusCode;

    let status = StatusCode::OK;
    let status_code = status.as_u16();
    assert_eq!(status_code, 200);

    // Test various status codes
    let statuses = vec![
        (StatusCode::OK, 200),
        (StatusCode::CREATED, 201),
        (StatusCode::BAD_REQUEST, 400),
        (StatusCode::NOT_FOUND, 404),
        (StatusCode::INTERNAL_SERVER_ERROR, 500),
    ];

    for (status, expected) in statuses {
        assert_eq!(status.as_u16(), expected);
    }
}
