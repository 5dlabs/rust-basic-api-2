use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

pub type DbPool = sqlx::PgPool;

/// Creates a database connection pool with production-ready configuration.
///
/// The pool is configured with:
/// - Connection limits (max and min connections)
/// - Timeout values for connection acquisition
/// - Idle timeout to recycle stale connections
/// - Max lifetime to prevent long-lived connections
///
/// # Errors
///
/// Returns `sqlx::Error` if the database URL is invalid or pool creation fails.
pub fn create_pool(database_url: &str, max_connections: u32) -> Result<DbPool, sqlx::Error> {
    let min_connections = if max_connections >= 2 {
        2
    } else {
        max_connections
    };

    PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect_lazy(database_url)
}

pub mod test_utils;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_pool_with_valid_url() {
        let url = "postgresql://localhost:5432/testdb";
        let pool = create_pool(url, 5);
        assert!(pool.is_ok());
        let pool = pool.unwrap();
        assert!(pool.size() == 0); // Lazy pool hasn't connected yet
    }

    #[tokio::test]
    async fn test_create_pool_with_custom_max_connections() {
        let url = "postgresql://localhost:5432/testdb";
        let pool = create_pool(url, 10);
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_create_pool_with_invalid_url() {
        let url = "invalid-url";
        let pool = create_pool(url, 5);
        assert!(pool.is_err());
    }

    #[tokio::test]
    async fn test_create_pool_with_minimal_connections() {
        let url = "postgresql://localhost:5432/testdb";
        let pool = create_pool(url, 1);
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_create_pool_with_high_connections() {
        let url = "postgresql://localhost:5432/testdb";
        let pool = create_pool(url, 100);
        assert!(pool.is_ok());
    }
}
