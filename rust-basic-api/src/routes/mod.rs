//! HTTP route definitions and handlers.

use axum::{extract::State, routing::get, Router};

use crate::error::AppResult;
use crate::repository::Database;

/// Shared application state injected into request handlers.
#[derive(Clone)]
pub struct AppState {
    /// Handle to the `PostgreSQL` connection pool.
    pub database: Database,
}

/// Construct the primary Axum router with all registered routes.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .with_state(state)
}

#[tracing::instrument(name = "health_check", skip(state))]
async fn health_check(State(state): State<AppState>) -> AppResult<&'static str> {
    state.database.is_healthy().await?;

    Ok("OK")
}
