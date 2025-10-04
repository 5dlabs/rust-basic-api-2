//! Database repositories live in this module.

use sqlx::PgPool;

/// Shared database abstraction to be expanded in subsequent tasks.
#[allow(dead_code)]
pub struct Repository {
    pool: PgPool,
}

#[allow(dead_code)]
impl Repository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
