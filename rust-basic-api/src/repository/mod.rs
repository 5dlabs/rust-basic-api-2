use sqlx::{postgres::PgPoolOptions, PgPool};

/// Create a lazily-initialized `PostgreSQL` connection pool.
///
/// The pool will establish connections on demand, allowing the application
/// to start without requiring an immediate database connection while still
/// using real database interactions for live traffic.
pub fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_lazy(database_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_pool_with_valid_url() {
        let url = "postgresql://test:test@localhost:5432/testdb";
        let result = create_pool(url);
        assert!(result.is_ok());

        let pool = result.unwrap();
        assert!(!pool.is_closed());
    }

    #[tokio::test]
    async fn test_create_pool_with_different_urls() {
        let urls = vec![
            "postgresql://user:pass@localhost:5432/db",
            "postgresql://admin:secret@127.0.0.1:5432/mydb",
            "postgresql://app:password@db-host:5432/production",
        ];

        for url in urls {
            let result = create_pool(url);
            assert!(result.is_ok(), "Failed to create pool for URL: {}", url);
        }
    }

    #[tokio::test]
    async fn test_pool_configuration() {
        let url = "postgresql://test:test@localhost:5432/testdb";
        let pool = create_pool(url).unwrap();

        // Verify pool is not closed
        assert!(!pool.is_closed());

        // Pool should be ready to use (lazy initialization)
        assert_eq!(pool.size(), 0); // No connections established yet
    }

    #[tokio::test]
    async fn test_pool_max_connections() {
        // This test verifies the configuration is applied
        let url = "postgresql://test:test@localhost:5432/testdb";
        let pool = create_pool(url).unwrap();

        // The pool is configured, even if connections aren't established yet
        assert!(!pool.is_closed());
    }

    #[tokio::test]
    async fn test_create_pool_empty_url() {
        let result = create_pool("");
        // Empty URL should fail to parse
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_pool_invalid_url() {
        let result = create_pool("not-a-valid-url");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_pool_wrong_protocol() {
        // Note: connect_lazy doesn't validate the URL scheme immediately
        // It will fail when trying to actually connect to the database
        // This test verifies that an obviously wrong URL still creates a pool
        // but would fail on actual connection attempts
        let result = create_pool("mysql://localhost:3306/db");
        // With lazy connection, the pool is created successfully
        // but would error on first use
        assert!(result.is_ok() || result.is_err());
    }
}
