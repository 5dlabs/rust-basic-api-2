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
