use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl From<ApiError> for axum::http::StatusCode {
    fn from(err: ApiError) -> Self {
        match err {
            ApiError::Serialization(_) => axum::http::StatusCode::BAD_REQUEST,
            ApiError::Config(_) | ApiError::Internal(_) => {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}
