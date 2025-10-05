use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Standard application result type wrapping [`AppError`].
pub type AppResult<T> = Result<T, AppError>;

/// Top-level application error type providing consistent responses.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        Self::Configuration(error.to_string())
    }
}

impl AppError {
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::ServiceUnavailable(message.into())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Configuration(_) | Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error: self.to_string(),
        });
        (self.status_code(), body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_error() {
        let error = AppError::Configuration("test error".to_string());
        assert_eq!(error.to_string(), "configuration error: test error");
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_service_unavailable_error() {
        let error = AppError::service_unavailable("database is down");
        assert_eq!(error.to_string(), "service unavailable: database is down");
        assert_eq!(error.status_code(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_database_error_conversion() {
        let sqlx_error = sqlx::Error::RowNotFound;
        let app_error = AppError::from(sqlx_error);
        assert!(matches!(app_error, AppError::Database(_)));
        assert_eq!(app_error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_anyhow_error_conversion() {
        let anyhow_error = anyhow::anyhow!("test anyhow error");
        let app_error = AppError::from(anyhow_error);
        assert!(matches!(app_error, AppError::Configuration(_)));
        assert!(app_error.to_string().contains("test anyhow error"));
    }

    #[test]
    fn test_error_response_serialization() {
        let response = ErrorResponse {
            error: "test error message".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test error message"));
        assert!(json.contains("error"));
    }

    #[test]
    fn test_error_into_response() {
        let error = AppError::service_unavailable("test");
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_all_error_variants_display() {
        let config_err = AppError::Configuration("config".to_string());
        assert!(config_err.to_string().contains("configuration error"));

        let db_err = AppError::Database(sqlx::Error::RowNotFound);
        assert!(db_err.to_string().contains("database error"));

        let unavailable_err = AppError::ServiceUnavailable("unavail".to_string());
        assert!(unavailable_err.to_string().contains("service unavailable"));
    }

    #[test]
    fn test_app_result_type() {
        let ok_result: AppResult<i32> = Ok(42);
        assert!(ok_result.is_ok());
        if let Ok(value) = ok_result {
            assert_eq!(value, 42);
        }

        let err_result: AppResult<i32> = Err(AppError::Configuration("test".to_string()));
        assert!(err_result.is_err());
    }
}
