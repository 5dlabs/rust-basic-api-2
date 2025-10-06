mod config;
mod error;
mod models;
mod repository;
mod routes;

use std::net::SocketAddr;

use anyhow::Context;
use routes::{create_router, AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env().context("failed to load configuration")?;
    let database = repository::Database::connect(&config)
        .context("failed to initialise database connection pool")?;

    let router = create_router(AppState { database });

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!(%addr, "listening on");

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server terminated unexpectedly")?;

    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        if let Err(error) = signal::ctrl_c().await {
            tracing::warn!(?error, "failed to install Ctrl+C handler");
        }
    };

    tokio::select! {
        () = ctrl_c => {
            tracing::info!("Ctrl+C received, shutting down");
        }
        () = terminate_signal() => {
            tracing::info!("Terminate signal received, shutting down");
        }
    }
}

#[cfg(unix)]
async fn terminate_signal() {
    use tokio::signal::unix::{signal, SignalKind};

    match signal(SignalKind::terminate()) {
        Ok(mut sig) => {
            sig.recv().await;
        }
        Err(error) => {
            tracing::warn!(?error, "failed to install SIGTERM handler");
        }
    }
}

#[cfg(not(unix))]
async fn terminate_signal() {
    std::future::pending::<()>().await;
}
