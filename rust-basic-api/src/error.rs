use std::fmt::Debug;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

use crate::config::ConfigError;

/// Unified application error type used across handlers and setup code.
#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!(error = %self, "request failed");
        let status = match &self {
            Self::Config(_) => StatusCode::BAD_REQUEST,
            Self::Database(_) | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(ErrorBody {
            error: self.to_string(),
        });
        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn config_errors_return_bad_request() {
        let error = ConfigError::MissingEnv("DATABASE_URL".into());
        let response = AppError::from(error).into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn other_errors_return_internal_server_error() {
        let error = anyhow::anyhow!("unexpected");
        let response = AppError::from(error).into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
