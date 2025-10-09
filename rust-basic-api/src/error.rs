use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let message = self.to_string();

        let status = match &self {
            Self::Configuration(_) | Self::Database(_) | Self::Unexpected(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let body = Json(json!({ "message": message }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
