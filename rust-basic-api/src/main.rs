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

async fn build_application(config: &Config) -> anyhow::Result<(SocketAddr, Router)> {
    let db_pool = create_pool(&config.database_url)
        .await
        .context("failed to initialize database connection pool")?;

    sqlx::migrate!()
        .run(&db_pool)
        .await
        .context("failed to run database migrations")?;

    tracing::info!("Database connected and migrations completed");

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
    let (address, router) = build_application(&config)
        .await
        .context("failed to build application components")?;
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
    use crate::config::database_url_from_env;
    use crate::repository::test_utils;
    use axum::{body::Body, http::Request, http::StatusCode};
    use serial_test::serial;
    use std::{env, io::Error, time::Duration};
    use tokio::{sync::oneshot, time::sleep};
    use tower::ServiceExt;

    struct EnvGuard {
        previous: Vec<(String, Option<std::ffi::OsString>)>,
    }

    impl EnvGuard {
        fn new(vars: &[(String, String)]) -> Self {
            let mut previous = Vec::with_capacity(vars.len());

            for (key, value) in vars {
                previous.push((key.clone(), env::var_os(key)));
                env::set_var(key, value);
            }

            Self { previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in self.previous.drain(..) {
                if let Some(existing) = value {
                    env::set_var(key, existing);
                } else {
                    env::remove_var(key);
                }
            }
        }
    }

    fn load_test_database_url() -> String {
        test_utils::ensure_test_env();
        database_url_from_env()
            .expect("configure TEST_DATABASE_URL, DATABASE_URL, or component variables for tests")
    }

    #[tokio::test]
    #[serial]
    async fn test_build_application_produces_router_and_address() {
        let database_url = load_test_database_url();
        let port = 3030u16;
        let _guard = EnvGuard::new(&[
            ("DATABASE_URL".to_string(), database_url.clone()),
            ("SERVER_PORT".to_string(), port.to_string()),
        ]);

        let pool = test_utils::setup_test_database()
            .await
            .expect("test database should initialize");
        test_utils::cleanup_database(&pool)
            .await
            .expect("cleanup should succeed");

        init_tracing();
        let config = Config::from_env().expect("config should load");
        let (address, router) = build_application(&config)
            .await
            .expect("application builds");

        assert_eq!(address.port(), port);

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
        let _guard = EnvGuard::new(&[
            ("DATABASE_URL".to_string(), "invalid-url".to_string()),
            ("SERVER_PORT".to_string(), "3031".to_string()),
        ]);
        let config = Config::from_env().expect("config should load");
        let result = build_application(&config).await;

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
        let database_url = load_test_database_url();
        let _guard = EnvGuard::new(&[
            ("DATABASE_URL".to_string(), database_url),
            ("SERVER_PORT".to_string(), "0".to_string()),
        ]);
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
