//! Error handling utilities for HTTP responses.

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

/// Convenient result alias for fallible HTTP handlers.
pub type AppResult<T> = Result<T, AppError>;

/// Application-level error type that can be converted into HTTP responses.
#[derive(Debug, Error)]
pub enum AppError {
    /// Wrapper around `SQLx` database errors.
    #[error("database error: {0}")]
    Database(sqlx::Error),
    /// Unexpected internal server error with contextual message.
    #[error("internal server error: {0}")]
    Internal(anyhow::Error),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::Database(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(ErrorResponse {
            error: self.to_string(),
        });

        (status, body).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        Self::Internal(error)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn test_app_error_database_display() {
        let error = AppError::Database(sqlx::Error::RowNotFound);
        let error_string = error.to_string();
        assert!(error_string.contains("database error"));
    }

    #[test]
    fn test_app_error_internal_display() {
        let error = AppError::Internal(anyhow!("test error"));
        let error_string = error.to_string();
        assert!(error_string.contains("internal server error"));
    }

    #[test]
    fn test_app_error_from_anyhow() {
        let anyhow_error = anyhow!("test anyhow error");
        let app_error: AppError = anyhow_error.into();

        match app_error {
            AppError::Internal(_) => {}
            AppError::Database(_) => panic!("Expected Internal error variant"),
        }
    }

    #[test]
    fn test_app_error_from_sqlx() {
        let sqlx_error = sqlx::Error::RowNotFound;
        let app_error: AppError = sqlx_error.into();

        match app_error {
            AppError::Database(_) => {}
            AppError::Internal(_) => panic!("Expected Database error variant"),
        }
    }

    #[test]
    fn test_app_error_into_response_database() {
        let error = AppError::Database(sqlx::Error::RowNotFound);
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_app_error_into_response_internal() {
        let error = AppError::Internal(anyhow!("test error"));
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_app_result_ok() {
        let result: AppResult<i32> = Ok(42);
        assert!(result.is_ok());
        if let Ok(value) = result {
            assert_eq!(value, 42);
        }
    }

    #[test]
    fn test_app_result_err() {
        let result: AppResult<i32> = Err(AppError::Internal(anyhow!("test")));
        assert!(result.is_err());
    }

    #[test]
    fn test_error_response_serialization() {
        let error_response = ErrorResponse {
            error: "test error".to_string(),
        };

        let json = serde_json::to_string(&error_response).unwrap();
        assert!(json.contains("test error"));
    }

    #[test]
    fn test_app_error_debug() {
        let error = AppError::Internal(anyhow!("debug test"));
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("Internal"));
    }
}
