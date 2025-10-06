use axum::{extract::State, routing::get, Router};
use tracing::{instrument, trace};

use crate::{error::AppResult, state::SharedAppState};

pub fn router() -> Router<SharedAppState> {
    Router::new().route("/health", get(health_check))
}

#[instrument(name = "routes.health", skip(state))]
async fn health_check(State(state): State<SharedAppState>) -> AppResult<&'static str> {
    trace!(
        has_database_url = !state.config.database_url.is_empty(),
        pool_closed = state.pool.is_closed(),
        "health check invoked"
    );
    Ok("OK")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, sync::Arc};

    use crate::{
        config::Config,
        repository::test_utils::{cleanup_database, setup_test_database},
        state::AppState,
    };
    use sqlx::query_scalar;

    fn default_database_url() -> String {
        let scheme = "postgresql";
        let user = "postgres";
        let password = "postgres";
        let host = "localhost";
        let port = 15432;
        let database = "rust_basic_api_test";

        format!("{scheme}://{user}:{password}@{host}:{port}/{database}")
    }

    fn database_url_from_env() -> String {
        env::var("DATABASE_URL").unwrap_or_else(|_| {
            let url = default_database_url();
            env::set_var("DATABASE_URL", &url);
            url
        })
    }

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
            database_url: database_url_from_env(),
            server_port: 3000,
        });

        let pool = setup_test_database().await;
        cleanup_database(&pool).await;

        let state = Arc::new(AppState::new(config, pool.clone()));

        let result = health_check(State(state)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "OK");

        cleanup_database(&pool).await;
    }

    #[tokio::test]
    async fn test_health_check_with_empty_database_url() {
        let config = Arc::new(Config {
            database_url: String::new(),
            server_port: 3000,
        });

        let pool = setup_test_database().await;
        cleanup_database(&pool).await;

        let state = Arc::new(AppState::new(config, pool.clone()));

        let result = health_check(State(state)).await;
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(response, "OK");
        }

        cleanup_database(&pool).await;
    }

    #[tokio::test]
    async fn test_health_check_multiple_calls() {
        let config = Arc::new(Config {
            database_url: database_url_from_env(),
            server_port: 3000,
        });

        let pool = setup_test_database().await;
        cleanup_database(&pool).await;

        let state = Arc::new(AppState::new(config, pool.clone()));

        for _ in 0..100 {
            let result = health_check(State(state.clone())).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "OK");
        }

        cleanup_database(&pool).await;
    }

    #[tokio::test]
    async fn test_app_state_type_alias() {
        let expected_url = database_url_from_env();
        let config = Arc::new(Config {
            database_url: expected_url.clone(),
            server_port: 3000,
        });

        let pool = setup_test_database().await;

        let state = AppState::new(config.clone(), pool.clone());

        assert_eq!(state.config.database_url, expected_url);
        assert_eq!(state.config.server_port, 3000);

        let mut connection = pool
            .acquire()
            .await
            .expect("should acquire connection from pool");
        let value: i32 = query_scalar("SELECT 1")
            .fetch_one(&mut connection)
            .await
            .expect("should execute simple query");
        assert_eq!(value, 1);

        cleanup_database(&pool).await;
    }

    #[tokio::test]
    async fn test_health_check_with_long_database_url() {
        let long_url = format!(
            "{scheme}://{user}:{password}@{host}:{port}/{database}?{params}",
            scheme = "postgresql",
            user = "user",
            password = "pass",
            host = "host",
            port = 5432,
            database = "db",
            params = "param=value&".repeat(100)
        );
        let config = Arc::new(Config {
            database_url: long_url,
            server_port: 3000,
        });

        let pool = setup_test_database().await;
        cleanup_database(&pool).await;

        let state = Arc::new(AppState::new(config, pool.clone()));

        let result = health_check(State(state)).await;
        assert!(result.is_ok());

        cleanup_database(&pool).await;
    }
}
