use axum::{Router, routing::get};

pub fn router() -> Router {
    Router::new().route("/health", get(health_check))
}

#[tracing::instrument(skip_all)]
async fn health_check() -> &'static str {
    "OK"
}
