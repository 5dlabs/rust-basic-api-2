//! Shared data models for the service.

use crate::repository::DbPool;

/// Shared application state made available to request handlers.
#[derive(Clone, Debug)]
pub struct AppState {
    /// Database connection pool used across the application.
    pub db_pool: DbPool,
}

impl AppState {
    /// Create a new [`AppState`] instance.
    #[must_use]
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }
}
