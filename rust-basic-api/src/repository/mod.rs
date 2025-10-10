use sqlx::postgres::PgPoolOptions;

pub type DbPool = sqlx::PgPool;

#[cfg(test)]
pub mod test_utils;

/// Creates a lazy database connection pool.
///
/// # Errors
///
/// Returns `sqlx::Error` if the database URL is invalid or pool creation fails.
pub fn create_pool(database_url: &str, max_connections: u32) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect_lazy(database_url)
}

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
