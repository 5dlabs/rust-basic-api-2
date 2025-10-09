use std::{
    future::{pending, Future},
    net::{Ipv4Addr, SocketAddr},
    sync::Once,
};

use axum::Router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{config::Config, error::AppResult, routes};

static TRACING: Once = Once::new();

/// Initialize the global tracing subscriber exactly once.
///
/// # Errors
///
/// This function never returns an error; the `Result` type is retained for API symmetry
/// with the rest of the application bootstrap pipeline.
pub fn init_tracing() -> AppResult<()> {
    TRACING.call_once(|| {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    });

    Ok(())
}

/// Build the HTTP router that powers the service.
pub fn build_router() -> Router {
    routes::router()
}

/// Resolve the socket address the server should bind to.
#[must_use]
pub fn bind_address(port: u16) -> SocketAddr {
    SocketAddr::from((Ipv4Addr::UNSPECIFIED, port))
}

/// Load configuration from the environment.
///
/// # Errors
///
/// Returns a [`ConfigError`](crate::error::ConfigError) when required environment variables are
/// missing, contain invalid unicode, or fail to parse into the expected type.
pub fn load_config() -> AppResult<Config> {
    Ok(Config::from_env()?)
}

/// Prepare the router, address, and configuration required to run the service using a supplied configuration.
pub fn bootstrap_with(config: Config) -> (Router, SocketAddr, Config) {
    let router = build_router();
    let address = bind_address(config.server_port);
    (router, address, config)
}

/// Prepare the router, address, and configuration required to run the service.
///
/// # Errors
///
/// Propagates configuration loading failures from [`load_config`].
pub fn bootstrap() -> AppResult<(Router, SocketAddr, Config)> {
    let config = load_config()?;
    Ok(bootstrap_with(config))
}

/// Launch the HTTP server using the supplied configuration.
///
/// # Errors
///
/// Returns an error if tracing initialization, configuration loading, or HTTP server startup
/// fails.
pub async fn run() -> AppResult<()> {
    let config = load_config()?;
    run_with_config(config, pending()).await
}

/// Run the HTTP server with a caller-provided shutdown signal and pre-built components.
///
/// # Errors
///
/// Returns an error if the underlying Axum server fails to bind or serve requests.
pub async fn run_with<S>(
    router: Router,
    address: SocketAddr,
    config: Config,
    shutdown: S,
) -> AppResult<()>
where
    S: Future<Output = ()> + Send + 'static,
{
    let has_database_credentials = !config.database_url.is_empty();
    tracing::debug!(has_database_credentials, "Loaded database configuration");
    tracing::info!(%address, "Listening on address");

    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await?;

    Ok(())
}

/// Launch the HTTP server with a supplied configuration and shutdown signal.
///
/// # Errors
///
/// Returns an error if tracing initialization or HTTP server execution fails.
pub async fn run_with_config<S>(config: Config, shutdown: S) -> AppResult<()>
where
    S: Future<Output = ()> + Send + 'static,
{
    init_tracing()?;
    let (router, address, config) = bootstrap_with(config);
    run_with(router, address, config, shutdown).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env,
        sync::{Mutex, OnceLock},
        time::Duration,
    };
    use tokio::{
        sync::oneshot,
        time::{sleep, timeout},
    };

    static ENV_GUARD: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        ENV_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("environment mutex poisoned")
    }

    #[test]
    fn tracing_initialization_is_idempotent() {
        init_tracing().expect("first initialization should succeed");
        init_tracing().expect("second initialization should be a no-op");
    }

    #[test]
    fn bind_address_uses_unspecified_ipv4() {
        let addr = bind_address(4_321);
        assert_eq!(addr.ip(), std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        assert_eq!(addr.port(), 4_321);
    }

    #[test]
    fn bootstrap_with_returns_expected_components() {
        let config = Config {
            database_url: "postgresql://localhost:5432/rust_basic_api".to_string(),
            server_port: 4_242,
        };

        let (_router, address, resulting_config) = bootstrap_with(config.clone());

        assert_eq!(address.port(), 4_242);
        assert_eq!(resulting_config.database_url, config.database_url);
    }

    #[tokio::test]
    async fn run_with_respects_shutdown_signal() {
        init_tracing().expect("tracing initialization should succeed");
        let router = build_router();
        let address = SocketAddr::from(([127, 0, 0, 1], 0));
        let config = Config {
            database_url: "postgresql://localhost:5432/rust_basic_api".to_string(),
            server_port: 0,
        };

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let server_handle = tokio::spawn(run_with(router, address, config, async move {
            let _ = shutdown_rx.await;
        }));

        sleep(Duration::from_millis(10)).await;
        shutdown_tx
            .send(())
            .expect("shutdown signal should be delivered");

        let server_result = server_handle.await.expect("server task");
        assert!(server_result.is_ok());
    }

    #[tokio::test]
    async fn run_with_config_respects_shutdown_signal() {
        let config = Config {
            database_url: "postgresql://localhost:5432/rust_basic_api".to_string(),
            server_port: 0,
        };

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let server_handle = tokio::spawn(run_with_config(config, async move {
            let _ = shutdown_rx.await;
        }));

        sleep(Duration::from_millis(10)).await;
        shutdown_tx
            .send(())
            .expect("shutdown signal should be delivered");

        let server_result = server_handle.await.expect("server task");
        assert!(server_result.is_ok());
    }

    #[tokio::test]
    #[allow(
        clippy::await_holding_lock,
        reason = "Ensures exclusive access to process environment while run() initializes."
    )]
    async fn run_future_can_be_aborted_after_startup() {
        let guard = env_guard();
        unsafe {
            // SAFETY: Access to environment variables is serialized via `env_guard`.
            env::set_var("DATABASE_URL", "postgresql://localhost:5432/rust_basic_api");
            env::set_var("SERVER_PORT", "0");
        }

        let result = timeout(Duration::from_millis(50), run()).await;
        assert!(result.is_err(), "run() should block until shutdown");

        unsafe {
            // SAFETY: Access to environment variables is serialized via `env_guard`.
            env::remove_var("DATABASE_URL");
            env::remove_var("SERVER_PORT");
        }
        drop(guard);
    }
}
