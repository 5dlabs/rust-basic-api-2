//! Database access layer for the application.

use sqlx::PgPool;

/// Shared database handle that can be cloned and passed to repositories.
#[derive(Clone, Debug)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new [`Database`] wrapper around an existing [`PgPool`].
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Access the inner [`PgPool`] reference.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    const TEST_DATABASE_URL: &str = "postgres://postgres@localhost:5432/test_db";

    #[tokio::test]
    async fn pool_accessor_provides_underlying_pg_pool() {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy(TEST_DATABASE_URL)
            .expect("lazy pool should be created");

        let database = Database::new(pool.clone());

        assert_eq!(database.pool().size(), pool.size());
    }
}
