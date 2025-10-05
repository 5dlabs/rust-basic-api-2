//! Application entry point.

pub mod config;
pub mod error;
pub mod models;
pub mod repository;
pub mod routes;

use std::net::SocketAddr;

use crate::repository::PoolSettings;
use config::Config;
use error::{AppError, AppResult};
use tokio::signal;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> AppResult<()> {
    init_tracing()?;

    let config = Config::from_env()?;

    let pool_settings = PoolSettings::from(&config.database);
    let pool = repository::create_pool(&config.database.url, &pool_settings)
        .await
        .map_err(|err| {
            warn!(error = %err, "failed to create database pool");
            AppError::from(err)
        })?;

    repository::MIGRATOR.run(&pool).await.map_err(|err| {
        warn!(error = %err, "failed to run database migrations");
        AppError::from(err)
    })?;

    info!("database connected and migrations completed");

    let app_state = routes::AppState { pool };
    let app = routes::create_router().with_state(app_state);

    let addr = SocketAddr::new(config.server_host, config.server_port);
    info!(%addr, "starting HTTP server");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|err| AppError::Runtime(err.into()))?;

    Ok(())
}

fn init_tracing() -> AppResult<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .map_err(|err| AppError::Runtime(err.into()))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = signal::ctrl_c().await {
            warn!(error = %err, "failed to listen for Ctrl+C");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};

        match signal(SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(err) => {
                warn!(error = %err, "failed to install SIGTERM handler");
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

    info!("shutdown signal received");
}
