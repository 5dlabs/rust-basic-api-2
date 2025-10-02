mod config;
mod error;
mod models;
mod repository;
mod routes;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Error as AnyhowError;
use error::{AppError, AppResult};
use models::{AppState, SharedState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::Config;

#[tokio::main]
async fn main() -> AppResult<()> {
    init_tracing();

    let config = Config::from_env()?;
    let state = build_state(&config)?;
    let router = routes::create_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!(listening_on = %addr, "Starting HTTP server");

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|error| AppError::from(AnyhowError::new(error)))?;

    Ok(())
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn build_state(config: &Config) -> AppResult<SharedState> {
    let pool = repository::create_pool(&config.database_url)?;
    Ok(Arc::new(AppState::new(pool)))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(?error, "Failed to listen for CTRL+C signal");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};

        match signal(SignalKind::terminate()) {
            Ok(mut term) => {
                term.recv().await;
            }
            Err(error) => {
                tracing::error!(?error, "Failed to install SIGTERM handler");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!(signal = "shutdown", "Shutting down gracefully");
}
