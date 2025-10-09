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
