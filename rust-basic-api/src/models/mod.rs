use std::sync::Arc;

use sqlx::PgPool;

/// Shared application state injected into request handlers.
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
}

impl AppState {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

pub type SharedState = Arc<AppState>;
