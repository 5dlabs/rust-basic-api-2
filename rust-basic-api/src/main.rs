mod config;
mod error;
mod models;
mod repository;
mod routes;

use anyhow::Context;
use axum::Router;
use config::Config;
use models::AppState;
use repository::create_pool;
use std::{future::Future, net::SocketAddr};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    start_application(shutdown_signal(async { tokio::signal::ctrl_c().await })).await
}

fn build_application(config: &Config) -> anyhow::Result<(SocketAddr, Router)> {
    let db_pool = create_pool(&config.database_url)
        .context("failed to initialize database connection pool")?;
    let state = AppState::new(db_pool);
    let router = routes::create_router(state);

    let address = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    Ok((address, router))
}

async fn start_application<F>(shutdown: F) -> anyhow::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    let config = Config::from_env().context("failed to load configuration")?;
    let (address, router) =
        build_application(&config).context("failed to build application components")?;
    tracing::info!(%address, "HTTP server listening");

    run_server(address, router, shutdown).await
}

fn init_tracing() {
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer());

    let _ = subscriber.try_init();
}

async fn run_server<F>(address: SocketAddr, router: Router, shutdown: F) -> anyhow::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await
        .context("server error")
}

async fn shutdown_signal<F>(signal: F)
where
    F: Future<Output = std::io::Result<()>> + Send,
{
    if let Err(error) = signal.await {
        tracing::error!(%error, "failed to listen for shutdown signal");
        return;
    }

    tracing::info!("Shutdown signal received, commencing graceful shutdown");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, http::StatusCode};
    use serial_test::serial;
    use std::{env, io::Error, time::Duration};
    use tokio::{sync::oneshot, time::sleep};
    use tower::ServiceExt;

    struct EnvGuard;

    impl EnvGuard {
        fn new(database_url: &str, port: &str) -> Self {
            env::set_var("DATABASE_URL", database_url);
            env::set_var("SERVER_PORT", port);
            Self
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            env::remove_var("DATABASE_URL");
            env::remove_var("SERVER_PORT");
        }
    }

    fn build_database_url() -> String {
        format!(
            "{scheme}://{user}:{pass}@{host}:{port}/{db}",
            scheme = "postgres",
            user = "postgres",
            pass = "postgres",
            host = "localhost",
            port = 5432,
            db = "postgres"
        )
    }

    #[tokio::test]
    #[serial]
    async fn test_build_application_produces_router_and_address() {
        let database_url = build_database_url();
        let _guard = EnvGuard::new(&database_url, "3030");
        init_tracing();
        let config = Config::from_env().expect("config should load");
        let (address, router) = build_application(&config).expect("application builds");

        assert_eq!(address.port(), 3030);

        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("valid request"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn test_build_application_rejects_invalid_database_url() {
        let _guard = EnvGuard::new("invalid-url", "3031");
        let config = Config::from_env().expect("config should load");
        let result = build_application(&config);

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_shutdown_signal_completes_after_trigger() {
        let (sender, receiver) = oneshot::channel();

        let signal = async move { receiver.await.map_err(|_| Error::other("channel closed")) };

        let handle = tokio::spawn(shutdown_signal(signal));

        sender.send(()).expect("signal send should succeed");
        handle.await.expect("shutdown task should complete");
    }

    #[tokio::test]
    async fn test_shutdown_signal_handles_error() {
        shutdown_signal(async { Err::<(), _>(Error::other("injected failure")) }).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_start_application_starts_and_stops() {
        let database_url = build_database_url();
        let _guard = EnvGuard::new(&database_url, "0");
        init_tracing();

        let (trigger, receiver) = oneshot::channel();
        let shutdown = async move {
            let _ = receiver.await;
        };

        let task = tokio::spawn(start_application(shutdown));

        sleep(Duration::from_millis(10)).await;
        trigger.send(()).expect("shutdown signal should send");

        task.await
            .expect("application task join should succeed")
            .expect("application should shut down cleanly");
    }
}
