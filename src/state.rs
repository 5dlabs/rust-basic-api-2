use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;

/// Shared application state distributed across request handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pool: PgPool,
}

impl AppState {
    #[must_use]
    pub fn new(config: Arc<Config>, pool: PgPool) -> Self {
        Self { config, pool }
    }
}

pub type SharedAppState = Arc<AppState>;
