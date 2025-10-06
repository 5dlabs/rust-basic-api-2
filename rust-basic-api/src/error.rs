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
