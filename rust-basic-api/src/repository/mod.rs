use std::time::Duration;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Pool, Postgres};

use crate::error::{AppError, Result};

pub type PgPool = Pool<Postgres>;

pub fn create_pool(database_url: &str) -> Result<PgPool> {
    let options: PgConnectOptions = database_url
        .parse()
        .map_err(|error| AppError::Configuration(format!("invalid DATABASE_URL value: {error}")))?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect_lazy_with(options);

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    async fn create_pool_from_valid_url() {
        let url = example_database_url();
        let pool = create_pool(&url).expect("pool should be created with valid URL");

        assert!(!pool.is_closed());
    }

    #[test]
    fn create_pool_from_invalid_url_returns_error() {
        let error = create_pool("not-a-valid-url").expect_err("invalid URL must error");

        match error {
            AppError::Configuration(message) => {
                assert!(message.contains("invalid DATABASE_URL"));
            }
            other => panic!("expected configuration error, got {other:?}"),
        }
    }
}
