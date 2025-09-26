use crate::{models::HealthResponse, repository::AppState};
use axum::{extract::State, http::StatusCode, Json};

pub async fn health_check(
    State(_state): State<AppState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let response = HealthResponse {
        status: "OK".to_string(),
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(response))
}
