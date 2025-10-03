//! Error handling utilities for the application.

use std::env;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

/// Convenient result alias that utilises [`AppError`].
pub type Result<T> = std::result::Result<T, AppError>;

/// Top-level application error type.
#[derive(Debug, Error)]
pub enum AppError {
    /// Configuration errors encountered during startup.
    #[error(transparent)]
    Configuration(#[from] ConfigError),

    /// Errors originating from the database layer.
    #[error(transparent)]
    Database(#[from] sqlx::Error),

    /// Any other error promoted to an application error.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        #[derive(Debug, serde::Serialize)]
        struct ErrorResponse {
            error: String,
        }

        let body = Json(ErrorResponse {
            error: self.to_string(),
        });

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

/// Errors that can occur while building the runtime configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// A required environment variable is missing.
    #[error("environment variable `{name}` is missing")]
    Missing { name: &'static str },

    /// Accessing an environment variable resulted in an error.
    #[error("failed to read environment variable `{name}`: {source}")]
    Environment {
        name: &'static str,
        #[source]
        source: env::VarError,
    },

    /// An environment variable contained an invalid number.
    #[error("failed to parse environment variable `{name}` with value `{value}`: {source}")]
    InvalidNumber {
        name: &'static str,
        value: String,
        #[source]
        source: std::num::ParseIntError,
    },
}
