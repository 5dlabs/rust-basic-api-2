use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

use crate::config::ConfigError;

/// Application-wide result type.
pub type AppResult<T> = Result<T, AppError>;

/// Common application error variants.
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error("server error: {0}")]
    Server(#[from] hyper::Error),

    #[error("tracing initialization error: {0}")]
    Tracing(#[from] tracing_subscriber::util::TryInitError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Config(_)
            | Self::Database(_)
            | Self::Server(_)
            | Self::Tracing(_)
            | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let payload = Json(json!({
            "error": self.to_string(),
        }));

        (status, payload).into_response()
    }
}
