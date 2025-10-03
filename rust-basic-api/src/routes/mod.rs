//! HTTP route handlers and router construction.

use axum::Extension;
use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;

use crate::models::AppState;

/// Build the primary application router.
pub fn create_router() -> Router {
    Router::new().route("/health", get(health_check))
}

async fn health_check(Extension(state): Extension<AppState>) -> (StatusCode, &'static str) {
    let _ = state.db_pool.clone();
    (StatusCode::OK, "OK")
}
