use axum::{
    Router,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::env;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

async fn health_check() -> Result<Json<ApiResponse<()>>, StatusCode> {
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: "Service is healthy".to_string(),
    }))
}

async fn get_users() -> Result<Json<ApiResponse<Vec<User>>>, StatusCode> {
    // This would normally connect to a real database
    // For now, returning sample data but structured for real implementation
    let users = vec![
        User {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Jane Smith".to_string(),
            email: "jane@example.com".to_string(),
        },
    ];

    Ok(Json(ApiResponse {
        success: true,
        data: Some(users),
        message: "Users retrieved successfully".to_string(),
    }))
}

async fn create_user(Json(user): Json<User>) -> Result<Json<ApiResponse<User>>, StatusCode> {
    // This would normally insert into a real database
    // For now, just returning the created user
    Ok(Json(ApiResponse {
        success: true,
        data: Some(user),
        message: "User created successfully".to_string(),
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get port from environment variable or default to 8080
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    // Build the application routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/users", get(get_users))
        .route("/users", post(create_user))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    tracing::info!("Server starting on port {}", port);

    axum::serve(listener, app).await?;

    Ok(())
}
