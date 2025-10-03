mod config;
mod error;
mod models;
mod repository;
mod routes;

use std::net::SocketAddr;
use std::sync::Arc;
#[cfg(test)]
use std::sync::Mutex;

use anyhow::Error as AnyhowError;
use error::{AppError, AppResult};
use models::{AppState, SharedState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::Config;

#[cfg(test)]
pub(crate) static ENV_LOCK: Mutex<()> = Mutex::new(());

#[tokio::main]
async fn main() -> AppResult<()> {
    init_tracing();

    let config = Config::from_env()?;
    run_application(config, shutdown_signal()).await
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer());

    if let Err(error) = subscriber.try_init() {
        tracing::warn!(?error, "Tracing subscriber already initialised");
    }
}

fn build_state(config: &Config) -> AppResult<SharedState> {
    let pool = repository::create_pool(&config.database_url)?;
    Ok(Arc::new(AppState::new(pool)))
}

async fn run_application(
    config: Config,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> AppResult<()> {
    let state = build_state(&config)?;
    let router = routes::create_router(state);

    let addr = SocketAddr::new(config.server_host, config.server_port);
    tracing::info!(%addr, "Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await
        .map_err(|error| AppError::from(AnyhowError::new(error)))?;

    Ok(())
}

async fn shutdown_signal() {
    wait_for_shutdown(ctrl_c_signal(), terminate_signal()).await;
}

async fn wait_for_shutdown(
    ctrl_c: impl std::future::Future<Output = ()> + Send,
    terminate: impl std::future::Future<Output = ()> + Send,
) {
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!(signal = "shutdown", "Shutting down gracefully");
}

async fn ctrl_c_signal() {
    #[cfg(not(test))]
    {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(?error, "Failed to listen for CTRL+C signal");
            std::future::pending::<()>().await;
        }
    }

    #[cfg(test)]
    {
        tokio::task::yield_now().await;
    }
}

async fn terminate_signal() {
    #[cfg(all(not(test), unix))]
    {
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
    }

    #[cfg(any(test, not(unix)))]
    {
        tokio::task::yield_now().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ENV_LOCK;
    use std::env;
    use std::net::IpAddr;

    #[tokio::test]
    async fn build_state_creates_shared_state() {
        let config = Config {
            database_url: "postgresql://localhost:5432/example_db".into(),
            server_host: IpAddr::from([0, 0, 0, 0]),
            server_port: 3000,
        };

        let state = build_state(&config).expect("state should be created");

        assert!(!state.db_pool.is_closed());
    }

    #[tokio::test]
    async fn run_application_resolves_when_shutdown_signalled() {
        init_tracing();
        let config = Config {
            database_url: "postgresql://localhost:5432/example_db".into(),
            server_host: IpAddr::from([0, 0, 0, 0]),
            server_port: 0,
        };

        run_application(config, async {})
            .await
            .expect("server should shut down");
    }

    #[tokio::test]
    async fn wait_for_shutdown_returns_when_ctrl_c_resolves() {
        init_tracing();
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        let ctrl_c = async {
            let _ = rx.await;
        };

        let terminate = std::future::pending::<()>();

        tokio::spawn(async move {
            let _ = tx.send(());
        });

        wait_for_shutdown(ctrl_c, terminate).await;
    }

    #[tokio::test]
    async fn shutdown_signal_completes_with_test_hooks() {
        init_tracing();
        tokio::time::timeout(std::time::Duration::from_millis(50), shutdown_signal())
            .await
            .expect("shutdown signal should resolve under test");
    }

    #[test]
    fn main_runs_with_environment_configuration() {
        let _guard = ENV_LOCK.lock().expect("mutex poisoned");

        env::set_var("DATABASE_URL", "postgresql://localhost:5432/example_db");
        env::set_var("SERVER_HOST", "0.0.0.0");
        env::set_var("SERVER_PORT", "0");

        super::main().expect("main should succeed");

        env::remove_var("SERVER_HOST");
        env::remove_var("SERVER_PORT");
        env::remove_var("DATABASE_URL");
    }
}
