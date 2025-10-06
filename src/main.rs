mod config;
mod error;
mod models;
mod repository;
mod routes;
mod state;

use std::{future::Future, net::SocketAddr, sync::Arc};

use anyhow::Context;
use axum::Router;
use config::Config;
use repository::create_pool;
use state::{AppState, SharedAppState};
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = Arc::new(Config::from_env()?);
    tracing::debug!(
        database_url_length = config.database_url.len(),
        "configuration loaded"
    );

    run_application(config, shutdown_signal()).await
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if let Err(error) = tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .try_init()
    {
        tracing::warn!(%error, "failed to initialize global tracing subscriber");
    }
}

fn build_router(state: SharedAppState) -> Router {
    routes::router().with_state(state)
}

async fn run_application(
    config: Arc<Config>,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let pool = create_pool(&config.database_url)
        .await
        .context("Failed to create database pool")?;

    sqlx::migrate!()
        .run(&pool)
        .await
        .context("Failed to run database migrations")?;

    tracing::info!("Database connected and migrations completed");

    let state: SharedAppState = Arc::new(AppState::new(config.clone(), pool));

    let router = build_router(state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("listening on {addr}");

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    match signal::ctrl_c().await {
        Ok(()) => tracing::info!("shutdown signal received"),
        Err(error) => tracing::error!(%error, "failed to listen for shutdown signal"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::test_utils::{cleanup_database, setup_test_database};
    use dotenv::from_filename;
    use serial_test::serial;
    use tokio::sync::oneshot;
    use tokio::time::{sleep, Duration};

    #[test]
    #[serial]
    fn test_init_tracing_multiple_calls() {
        // First call should succeed
        init_tracing();
        // Second call should not panic (will log a warning)
        init_tracing();
    }

    #[tokio::test]
    async fn test_build_router_creates_router() {
        let config = Arc::new(Config {
            database_url: "postgresql://localhost/testdb".to_string(),
            server_port: 3000,
        });

        let pool = setup_test_database().await;
        cleanup_database(&pool).await;

        let state: SharedAppState = Arc::new(AppState::new(config, pool.clone()));

        let router = build_router(state);
        // If this compiles and runs, router is created successfully
        assert!(std::mem::size_of_val(&router) > 0);

        cleanup_database(&pool).await;
    }

    #[tokio::test]
    async fn test_build_router_with_different_configs() {
        let config1 = Arc::new(Config {
            database_url: "postgresql://localhost/db1".to_string(),
            server_port: 3000,
        });

        let config2 = Arc::new(Config {
            database_url: "postgresql://localhost/db2".to_string(),
            server_port: 8080,
        });

        let pool = setup_test_database().await;
        cleanup_database(&pool).await;

        let state1: SharedAppState = Arc::new(AppState::new(config1, pool.clone()));
        let state2: SharedAppState = Arc::new(AppState::new(config2, pool.clone()));

        let _router1 = build_router(state1);
        let _router2 = build_router(state2);
        // Both routers should be created successfully

        cleanup_database(&pool).await;
    }

    #[tokio::test]
    async fn test_run_application_immediate_shutdown() {
        from_filename(".env.test").ok();
        std::env::set_var("SERVER_PORT", "0");

        let config = Arc::new(Config::from_env().expect("config should load"));

        let (tx, rx) = oneshot::channel::<()>();

        let shutdown_future = async move {
            let _ = rx.await;
        };

        let handle = tokio::spawn(run_application(config, shutdown_future));

        tx.send(()).expect("shutdown signal should send");

        let result = handle
            .await
            .expect("run_application task should complete successfully");

        assert!(result.is_ok());

        std::env::remove_var("SERVER_PORT");
    }

    #[tokio::test]
    async fn test_shutdown_signal_handles_ctrl_c() {
        let shutdown = tokio::spawn(async {
            tokio::time::timeout(Duration::from_secs(2), shutdown_signal())
                .await
                .expect("shutdown should complete");
        });

        sleep(Duration::from_millis(100)).await;

        unsafe {
            libc::raise(libc::SIGINT);
        }

        shutdown.await.expect("task should join successfully");
    }

    #[test]
    fn test_socket_addr_creation() {
        let port = 3000_u16;
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        assert_eq!(addr.port(), 3000);
        assert_eq!(addr.ip().to_string(), "0.0.0.0");
    }

    #[test]
    fn test_socket_addr_with_different_ports() {
        let ports = [3000, 8080, 9000, 5000];
        for port in ports {
            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            assert_eq!(addr.port(), port);
        }
    }

    #[tokio::test]
    async fn test_config_arc_sharing() {
        let config = Arc::new(Config {
            database_url: "postgresql://localhost/testdb".to_string(),
            server_port: 3000,
        });

        let config_clone = config.clone();
        assert_eq!(config.database_url, config_clone.database_url);
        assert_eq!(config.server_port, config_clone.server_port);
        assert_eq!(Arc::strong_count(&config), 2);
    }

    #[test]
    #[serial]
    fn test_tracing_initialization_with_env_filter() {
        // Set RUST_LOG environment variable
        std::env::set_var("RUST_LOG", "debug");
        init_tracing();
        std::env::remove_var("RUST_LOG");
    }

    #[test]
    #[serial]
    fn test_env_filter_defaults_to_info() {
        std::env::remove_var("RUST_LOG");
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let debug_str = format!("{env_filter:?}");
        // The filter should contain "info" somewhere in its debug representation
        assert!(
            debug_str.contains("info") || debug_str.contains("INFO"),
            "EnvFilter debug output should contain 'info' or 'INFO': {debug_str}"
        );
    }
}
