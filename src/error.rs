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
