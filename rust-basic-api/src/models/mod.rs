use sqlx::PgPool;

/// Shared application state made available to request handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

impl AppState {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn test_app_state_new() {
        // Create a lazy pool for testing (doesn't require actual DB connection)
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgresql://test:test@localhost:5432/test")
            .expect("Failed to create test pool");

        let state = AppState::new(pool);
        assert!(!state.pool.is_closed());
    }

    #[tokio::test]
    async fn test_app_state_clone() {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgresql://test:test@localhost:5432/test")
            .expect("Failed to create test pool");

        let state = AppState::new(pool);
        let cloned = state.clone();

        // Both states should reference the same pool
        assert!(!state.pool.is_closed());
        assert!(!cloned.pool.is_closed());
    }

    #[tokio::test]
    async fn test_app_state_db_pool_access() {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy("postgresql://test:test@localhost:5432/test")
            .expect("Failed to create test pool");

        let state = AppState::new(pool.clone());

        // Verify we can access the pool through the state
        assert_eq!(state.pool.size(), pool.size());
    }
}
