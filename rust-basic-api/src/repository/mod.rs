//! Database interaction layer for the application.

use sqlx::PgPool;

/// Convenience alias for repository operations across the service.
pub type RepositoryPool = PgPool;
