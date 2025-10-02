use axum::{extract::State, routing::get, Router};

use crate::models::SharedState;

pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .with_state(state)
}

async fn health_check(State(state): State<SharedState>) -> &'static str {
    if state.db_pool.is_closed() {
        tracing::warn!("database" = "closed", "Connection pool is closed");
    }

    "OK"
}
