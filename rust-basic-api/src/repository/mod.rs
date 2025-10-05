//! Database interaction layer for the application.

use std::time::Duration;

use sqlx::{migrate::Migrator, postgres::PgPoolOptions, Error, PgPool};
use tokio::time::timeout;

use crate::config::DatabaseSettings;

/// Convenience alias for repository operations across the service.
pub type RepositoryPool = PgPool;

/// Embedded migrator that loads SQL scripts from `./migrations` at compile time.
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Pool tuning parameters controlling how `sqlx` interacts with `PostgreSQL`.
#[derive(Debug, Clone)]
pub struct PoolSettings {
    /// Maximum number of active connections managed by the pool.
    pub max_connections: u32,
    /// Minimum number of connections held idle and ready to serve.
    pub min_connections: u32,
    /// Maximum time to wait for a free pooled connection.
    pub acquire_timeout: Duration,
    /// Maximum time to wait while establishing a brand-new `PostgreSQL` connection.
    pub connect_timeout: Duration,
    /// Maximum duration an idle connection can live before being closed.
    pub idle_timeout: Option<Duration>,
    /// Maximum lifetime permitted for any single connection.
    pub max_lifetime: Option<Duration>,
}

impl From<&DatabaseSettings> for PoolSettings {
    fn from(settings: &DatabaseSettings) -> Self {
        Self {
            max_connections: settings.max_connections,
            min_connections: settings.min_connections,
            acquire_timeout: settings.acquire_timeout,
            connect_timeout: settings.connect_timeout,
            idle_timeout: settings.idle_timeout,
            max_lifetime: settings.max_lifetime,
        }
    }
}

/// Create a new `PostgreSQL` pool using the supplied configuration.
///
/// # Errors
///
/// Returns an error if the pool fails to connect to the database using the
/// provided connection string or the configuration is otherwise invalid.
pub async fn create_pool(
    database_url: &str,
    settings: &PoolSettings,
) -> Result<RepositoryPool, sqlx::Error> {
    let mut options = PgPoolOptions::new();
    options = options
        .max_connections(settings.max_connections)
        .min_connections(settings.min_connections)
        .acquire_timeout(settings.acquire_timeout);

    if let Some(idle_timeout) = settings.idle_timeout {
        options = options.idle_timeout(idle_timeout);
    }

    if let Some(max_lifetime) = settings.max_lifetime {
        options = options.max_lifetime(max_lifetime);
    }

    let connect_future = options.connect(database_url);
    match timeout(settings.connect_timeout, connect_future).await {
        Ok(pool_result) => pool_result,
        Err(_) => Err(Error::PoolTimedOut),
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;
