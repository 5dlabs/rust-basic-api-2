//! HTTP route handlers and router composition.

pub mod health;

use axum::{routing::get, Router};

use crate::repository::Database;

/// Shared application state injected into request handlers.
#[derive(Clone, Debug)]
pub struct AppState {
    pub database: Database,
}

impl AppState {
    /// Create a new state value.
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

/// Compose the application's router tree.
pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health::health_check))
}
