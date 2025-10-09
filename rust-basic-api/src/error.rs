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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    async fn extract_body(response: Response) -> serde_json::Value {
        let bytes = hyper::body::to_bytes(response.into_body())
            .await
            .expect("body should convert to bytes");
        serde_json::from_slice(&bytes).expect("body should be valid json")
    }

    #[tokio::test]
    async fn configuration_error_into_response() {
        let response = AppError::Configuration("missing".into()).into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = extract_body(response).await;
        assert_eq!(body["message"], "configuration error: missing");
    }

    #[tokio::test]
    async fn database_error_into_response() {
        let db_error = sqlx::Error::ColumnNotFound("users".into());
        let response = AppError::Database(db_error).into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = extract_body(response).await;
        assert!(!body["message"].as_str().unwrap_or_default().is_empty());
    }

    #[tokio::test]
    async fn unexpected_error_into_response() {
        let response = AppError::Unexpected(anyhow::anyhow!("boom")).into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = extract_body(response).await;
        assert_eq!(body["message"], "boom");
    }
}
