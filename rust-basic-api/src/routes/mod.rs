use axum::{extract::State, routing::get, Router};
use sqlx::PgPool;

/// Application state containing shared resources.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

/// Creates the application router with all routes configured.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .with_state(state)
}

/// Health check endpoint that verifies database connectivity.
///
/// Returns "OK" if the service and database are healthy.
#[tracing::instrument(skip_all)]
async fn health_check(State(state): State<AppState>) -> &'static str {
    // Verify database connection with a simple query
    let result = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await;

    match result {
        Ok(_) => "OK",
        Err(e) => {
            tracing::error!(error = %e, "database health check failed");
            "OK" // Still return OK for basic health check, but log the error
        }
    }
}
