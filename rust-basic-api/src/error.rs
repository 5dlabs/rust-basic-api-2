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
