use anyhow::Context;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::env;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

#[derive(Serialize, Deserialize, FromRow, Clone)]
struct User {
    id: i32,
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Clone)]
struct AppState {
    db: PgPool,
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

async fn get_users(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<User>>>, StatusCode> {
    let users = sqlx::query_as::<_, User>("SELECT id, name, email FROM users ORDER BY id")
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch users: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(users),
        message: "Users retrieved successfully".to_string(),
    }))
}

async fn create_user(
    State(state): State<AppState>,
    Json(create_user): Json<CreateUser>,
) -> Result<Json<ApiResponse<User>>, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email"
    )
    .bind(create_user.name)
    .bind(create_user.email)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create user: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

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

    // Get configuration from environment
    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL environment variable is required")?;

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .context("Invalid PORT value")?;

    // Connect to database
    tracing::info!("Connecting to database...");
    let db = PgPool::connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    // Run database migrations (in a real app, you'd use sqlx migrate)
    tracing::info!("Setting up database schema...");
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        )
        ",
    )
    .execute(&db)
    .await
    .context("Failed to create users table")?;

    let state = AppState { db };

    // Build the application routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/users", get(get_users))
        .route("/users", post(create_user))
        .with_state(state)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .context("Failed to bind to address")?;

    tracing::info!("Server starting on port {} with database connection", port);

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}
