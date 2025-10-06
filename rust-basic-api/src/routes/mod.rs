//! HTTP route definitions and handlers.

use axum::{extract::State, routing::get, Router};
use sqlx::PgPool;

use crate::error::AppResult;

/// Shared application state injected into request handlers.
#[derive(Clone)]
pub struct AppState {
    /// Handle to the `PostgreSQL` connection pool.
    pub pool: PgPool,
}

/// Construct the primary Axum router with all registered routes.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .with_state(state)
}

#[tracing::instrument(name = "health_check", skip(state))]
async fn health_check(State(state): State<AppState>) -> AppResult<&'static str> {
    sqlx::query("SELECT 1").execute(&state.pool).await?;

    Ok("OK")
}
