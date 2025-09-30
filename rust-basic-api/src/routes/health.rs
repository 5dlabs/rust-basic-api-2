use axum::{extract::State, http::StatusCode, response::IntoResponse};
use tracing::info;

use super::AppState;

/// Simple health check endpoint used by load balancers and uptime monitors.
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let pool_size = state.database.pool().size();
    info!(connections = pool_size, "health check invoked");
    (StatusCode::OK, "OK")
}
