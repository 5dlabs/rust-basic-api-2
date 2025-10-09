use sqlx::postgres::PgPoolOptions;

pub type DbPool = sqlx::PgPool;

pub fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(database_url)
}
