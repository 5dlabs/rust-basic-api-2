mod config;
mod error;
mod models;
mod repository;
mod routes;

use axum::Router;
use std::net::SocketAddr;

use anyhow::{Context, Result};
use config::Config;
use repository::Database;
use routes::{router, AppState};
use sqlx::{postgres::PgPoolOptions, PgPool};
#[cfg(not(test))]
use tokio::signal;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    start_application(shutdown_signal()).await
}

#[cfg(not(test))]
async fn shutdown_signal() {
    if let Err(error) = signal::ctrl_c().await {
        tracing::warn!(%error, "failed to listen for shutdown signal");
    }
}

#[cfg(test)]
#[allow(clippy::unused_async)]
async fn shutdown_signal() {}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let init_result = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .try_init();

    if let Err(error) = init_result {
        tracing::trace!(%error, "tracing already initialized, continuing");
    }
}

fn create_database_pool(database_url: &str, max_connections: u32) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect_lazy(database_url)
        .context("failed to initialize database pool")
}

fn build_app(state: AppState) -> Router<()> {
    router().with_state(state)
}

fn bind_address(port: u16) -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], port))
}

async fn run_with_config(
    config: Config,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> Result<()> {
    let pool = create_database_pool(&config.database_url, config.database_max_connections)?;
    let state = AppState::new(Database::new(pool));
    let app = build_app(state);
    let addr = bind_address(config.server_port);
    info!(%addr, "Starting HTTP server");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await
        .context("server error")
}

async fn start_application(
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> Result<()> {
    init_tracing();
    let config = Config::from_env().map_err(anyhow::Error::new)?;
    run_with_config(config, shutdown).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use hyper::{body::to_bytes, Client, Uri};
    use serial_test::serial;
    use std::env;
    use std::net::TcpListener;
    use std::time::Duration;
    use tokio::sync::oneshot;
    use tokio::time::timeout;
    use tower::ServiceExt;

    const TEST_DATABASE_URL: &str = "postgres://postgres@localhost:5432/test_db";

    #[test]
    fn bind_address_uses_requested_port() {
        let addr = bind_address(8080);
        assert_eq!(addr.port(), 8080);
        assert_eq!(addr.ip().to_string(), "0.0.0.0");
    }

    #[test]
    #[serial]
    fn init_tracing_allows_multiple_calls() {
        init_tracing();
        init_tracing();
    }

    #[tokio::test]
    async fn create_database_pool_accepts_valid_url() {
        let pool =
            create_database_pool(TEST_DATABASE_URL, 5).expect("pool should be created lazily");
        assert_eq!(pool.size(), 0_u32);
    }

    #[tokio::test]
    async fn build_app_registers_health_route() {
        let pool =
            create_database_pool(TEST_DATABASE_URL, 5).expect("pool should be created lazily");
        let database = Database::new(pool);
        let state = AppState::new(database);
        let app = build_app(state);

        let request = Request::builder()
            .uri("/health")
            .method("GET")
            .body(Body::empty())
            .expect("failed to build request");

        let response = app.oneshot(request).await.expect("request should succeed");

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = to_bytes(response.into_body())
            .await
            .expect("body available");
        assert_eq!(body, "OK");
    }

    #[tokio::test]
    async fn create_database_pool_rejects_invalid_url() {
        let error =
            create_database_pool("not-a-valid-url", 5).expect_err("pool creation should fail");
        assert!(error
            .to_string()
            .contains("failed to initialize database pool"));
    }

    #[tokio::test]
    async fn run_with_config_serves_requests_and_honors_shutdown() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        let port = listener.local_addr().expect("listener addr").port();
        drop(listener);

        let config = Config {
            database_url: TEST_DATABASE_URL.to_string(),
            server_port: port,
            database_max_connections: 5,
        };

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let server_task = tokio::spawn(run_with_config(config.clone(), async {
            let _ = shutdown_rx.await;
        }));

        // Allow time for the server to start listening.
        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = Client::new();
        let uri: Uri = format!("http://127.0.0.1:{port}/health")
            .parse()
            .expect("valid uri");

        let response = timeout(Duration::from_secs(2), client.get(uri))
            .await
            .expect("request completed")
            .expect("request succeeded");

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let _ = shutdown_tx.send(());

        server_task
            .await
            .expect("server task should join")
            .expect("server should exit cleanly");
    }

    #[tokio::test]
    #[serial]
    async fn start_application_uses_environment_configuration() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        let port = listener.local_addr().expect("listener addr").port();
        drop(listener);

        env::set_var("DATABASE_URL", TEST_DATABASE_URL);
        env::set_var("SERVER_PORT", port.to_string());
        env::set_var("DATABASE_MAX_CONNECTIONS", "5");

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let server_task = tokio::spawn(start_application(async {
            let _ = shutdown_rx.await;
        }));

        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = Client::new();
        let uri: Uri = format!("http://127.0.0.1:{port}/health")
            .parse()
            .expect("valid uri");

        let response = timeout(Duration::from_secs(2), client.get(uri))
            .await
            .expect("request completed")
            .expect("request succeeded");

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let _ = shutdown_tx.send(());

        server_task
            .await
            .expect("server task should join")
            .expect("server should exit cleanly");

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
        env::remove_var("DATABASE_MAX_CONNECTIONS");
    }

    #[test]
    #[serial]
    fn main_initializes_and_exits_with_immediate_shutdown() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        let port = listener.local_addr().expect("listener addr").port();
        drop(listener);

        env::set_var("DATABASE_URL", TEST_DATABASE_URL);
        env::set_var("SERVER_PORT", port.to_string());
        env::set_var("DATABASE_MAX_CONNECTIONS", "5");

        main().expect("main should exit cleanly");

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
        env::remove_var("DATABASE_MAX_CONNECTIONS");
    }
}
