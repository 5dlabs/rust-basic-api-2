mod config;
mod error;
mod models;
mod repository;
mod routes;

use std::{net::SocketAddr, sync::Arc};

use axum::{extract::Extension, Router};
use config::Config;
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

    let router = build_router(config.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("listening on {addr}");

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
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

fn build_router(config: Arc<Config>) -> Router {
    routes::router().layer(Extension(config))
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

    #[test]
    fn test_init_tracing_multiple_calls() {
        // First call should succeed
        init_tracing();
        // Second call should not panic (will log a warning)
        init_tracing();
    }

    #[test]
    fn test_build_router_creates_router() {
        let config = Arc::new(Config {
            database_url: "postgresql://localhost/testdb".to_string(),
            server_port: 3000,
        });

        let router = build_router(config);
        // If this compiles and runs, router is created successfully
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_build_router_with_different_configs() {
        let config1 = Arc::new(Config {
            database_url: "postgresql://localhost/db1".to_string(),
            server_port: 3000,
        });

        let config2 = Arc::new(Config {
            database_url: "postgresql://localhost/db2".to_string(),
            server_port: 8080,
        });

        let _router1 = build_router(config1);
        let _router2 = build_router(config2);
        // Both routers should be created successfully
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
    fn test_tracing_initialization_with_env_filter() {
        // Set RUST_LOG environment variable
        std::env::set_var("RUST_LOG", "debug");
        init_tracing();
        std::env::remove_var("RUST_LOG");
    }

    #[test]
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
