use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

use crate::config::ConfigError;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("configuration error: {0}")]
    Configuration(#[from] ConfigError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        error!(target: "api::error", error = %self, "request failed");

        let status = match self {
            Self::Database(_) | Self::Unexpected(_) | Self::Configuration(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let body = Json(ErrorResponse {
            error: "Internal server error".to_string(),
        });

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn test_app_error_from_config_error() {
        let config_error = ConfigError::MissingEnvironmentVariable("TEST");
        let app_error: AppError = config_error.into();

        assert!(matches!(app_error, AppError::Configuration(_)));
        assert!(format!("{app_error}").contains("configuration error"));
    }

    #[test]
    fn test_app_error_from_anyhow() {
        let anyhow_error = anyhow!("test error");
        let app_error: AppError = anyhow_error.into();

        assert!(matches!(app_error, AppError::Unexpected(_)));
    }

    #[test]
    fn test_app_error_display() {
        let config_error = ConfigError::MissingEnvironmentVariable("DATABASE_URL");
        let app_error = AppError::Configuration(config_error);

        let error_msg = format!("{app_error}");
        assert!(error_msg.contains("configuration error"));
        assert!(error_msg.contains("DATABASE_URL"));
    }

    #[test]
    fn test_app_error_debug() {
        let config_error = ConfigError::MissingEnvironmentVariable("TEST");
        let app_error = AppError::Configuration(config_error);

        let debug_str = format!("{app_error:?}");
        assert!(debug_str.contains("Configuration"));
    }

    #[test]
    fn test_error_response_into_response() {
        let config_error = ConfigError::MissingEnvironmentVariable("TEST");
        let app_error = AppError::Configuration(config_error);

        let response = app_error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_unexpected_error_into_response() {
        let anyhow_error = anyhow!("unexpected error");
        let app_error = AppError::Unexpected(anyhow_error);

        let response = app_error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_response_serialization() {
        let error_response = ErrorResponse {
            error: "Test error".to_string(),
        };

        let json = serde_json::to_string(&error_response).unwrap();
        assert!(json.contains("Test error"));
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
        let config_error = ConfigError::MissingEnvironmentVariable("TEST");
        let result: AppResult<i32> = Err(AppError::Configuration(config_error));
        assert!(result.is_err());
    }
}
