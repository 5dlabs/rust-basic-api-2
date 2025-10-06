use std::sync::Arc;

use axum::{extract::Extension, routing::get, Router};
use tracing::{instrument, trace};

use crate::{config::Config, error::AppResult};

pub type AppState = Arc<Config>;

pub fn router() -> Router {
    Router::new().route("/health", get(health_check))
}

#[instrument(name = "routes.health", skip(config))]
async fn health_check(Extension(config): Extension<AppState>) -> AppResult<&'static str> {
    trace!(
        has_database_url = !config.database_url.is_empty(),
        "health check invoked"
    );
    Ok("OK")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let router = router();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_router_is_cloneable() {
        let router1 = router();
        let _router2 = router1.clone();
    }

    #[tokio::test]
    async fn test_health_check_with_valid_config() {
        let config = Arc::new(Config {
            database_url: "postgresql://localhost/testdb".to_string(),
            server_port: 3000,
        });

        let result = health_check(Extension(config)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "OK");
    }

    #[tokio::test]
    async fn test_health_check_with_empty_database_url() {
        let config = Arc::new(Config {
            database_url: String::new(),
            server_port: 3000,
        });

        let result = health_check(Extension(config)).await;
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(response, "OK");
        }
    }

    #[tokio::test]
    async fn test_health_check_multiple_calls() {
        let config = Arc::new(Config {
            database_url: "postgresql://localhost/testdb".to_string(),
            server_port: 3000,
        });

        for _ in 0..100 {
            let result = health_check(Extension(config.clone())).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "OK");
        }
    }

    #[test]
    fn test_app_state_type_alias() {
        let config: AppState = Arc::new(Config {
            database_url: "postgresql://localhost/testdb".to_string(),
            server_port: 3000,
        });

        assert_eq!(config.database_url, "postgresql://localhost/testdb");
        assert_eq!(config.server_port, 3000);
    }

    #[tokio::test]
    async fn test_health_check_with_long_database_url() {
        let long_url = format!(
            "postgresql://user:pass@host:5432/db?{}",
            "param=value&".repeat(100)
        );
        let config = Arc::new(Config {
            database_url: long_url,
            server_port: 3000,
        });

        let result = health_check(Extension(config)).await;
        assert!(result.is_ok());
    }
}
