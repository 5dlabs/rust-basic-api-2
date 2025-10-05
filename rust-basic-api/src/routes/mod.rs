use axum::{Router, extract::State, http::StatusCode, routing::get};

use crate::{
    error::{AppError, AppResult},
    models::AppState,
};

/// Construct the application router with all public routes.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .with_state(state)
}

async fn health_check(State(state): State<AppState>) -> AppResult<(StatusCode, &'static str)> {
    if state.db_pool.is_closed() {
        tracing::warn!("Database pool reported as closed during health check");
        return Err(AppError::service_unavailable("database pool is closed"));
    }

    Ok((StatusCode::OK, "OK"))
}
