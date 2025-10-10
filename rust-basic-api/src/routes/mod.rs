use axum::{routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/health", get(health_check))
}

#[tracing::instrument(skip_all)]
async fn health_check() -> &'static str {
    "OK"
}
