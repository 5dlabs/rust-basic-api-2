use std::{future::Future, net::SocketAddr};

use anyhow::Context;
use axum::{Router, Server};

use crate::{
    config::Config,
    repository,
    routes::{self, AppState},
};

pub struct Application {
    addr: SocketAddr,
    router: Router,
}

impl Application {
    pub fn build() -> anyhow::Result<Self> {
        init_tracing();

        let config = Config::from_env()?;
        let db_pool = repository::create_pool(&config.database_url)
            .context("failed to initialize database connection pool")?;

        let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
        let router = routes::create_router(AppState { db_pool });

        Ok(Self { addr, router })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        self.run_until(shutdown_signal()).await
    }

    pub async fn run_until<F>(self, shutdown: F) -> anyhow::Result<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let Application { addr, router } = self;

        tracing::info!("Listening on {}", addr);

        Server::bind(&addr)
            .serve(router.into_make_service())
            .with_graceful_shutdown(shutdown)
            .await
            .context("server encountered an unrecoverable error")
    }
}

fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let registry = tracing_subscriber::registry().with(tracing_subscriber::EnvFilter::new(
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
    ));

    let _ = registry.with(tracing_subscriber::fmt::layer()).try_init();
}

#[cfg(not(test))]
async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::warn!(%error, "failed to listen for shutdown signal");
    }
}

#[cfg(test)]
fn shutdown_signal() -> impl Future<Output = ()> + Send + 'static {
    std::future::ready(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, http::StatusCode};
    use serial_test::serial;
    use std::env;
    use tokio::time::{sleep, timeout, Duration};
    use tower::ServiceExt;

    fn clear_env() {
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
        env::remove_var("RUST_LOG");
    }

    fn example_database_url() -> String {
        format!(
            "{scheme}://{user}:{password}@{host}:{port}/{database}",
            scheme = "postgres",
            user = "example_user",
            password = "example_secret",
            host = "localhost",
            port = 5432,
            database = "example_db"
        )
    }

    #[tokio::test]
    #[serial]
    async fn build_application_uses_environment_configuration() {
        clear_env();
        env::set_var("DATABASE_URL", example_database_url());
        env::set_var("SERVER_PORT", "4100");
        env::set_var("RUST_LOG", "debug");

        let application =
            Application::build().expect("application should build from environment variables");

        assert_eq!(application.addr.port(), 4100);

        let response = application
            .router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("health request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        clear_env();
    }

    #[tokio::test]
    #[serial]
    async fn run_until_respects_shutdown_signal() {
        clear_env();
        env::set_var("DATABASE_URL", example_database_url());
        env::set_var("SERVER_PORT", "0");

        let application =
            Application::build().expect("application should build from environment variables");

        let handle = tokio::spawn(application.run_until(async {
            sleep(Duration::from_millis(50)).await;
        }));

        let result = timeout(Duration::from_secs(1), handle)
            .await
            .expect("server should shut down within timeout")
            .expect("run_until task should not panic");

        assert!(result.is_ok());

        clear_env();
    }

    #[tokio::test]
    #[serial]
    async fn run_uses_default_shutdown() {
        clear_env();
        env::set_var("DATABASE_URL", example_database_url());
        env::set_var("SERVER_PORT", "0");

        let application =
            Application::build().expect("application should build from environment variables");

        let result = application.run().await;

        assert!(result.is_ok());

        clear_env();
    }
}
