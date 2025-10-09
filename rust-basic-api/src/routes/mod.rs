use axum::{extract::State, routing::get, Router};

use crate::repository::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .with_state(state)
}

async fn health_check(State(state): State<AppState>) -> &'static str {
    if state.db_pool.is_closed() {
        tracing::warn!("database connection pool is closed");
    }

    "OK"
}
