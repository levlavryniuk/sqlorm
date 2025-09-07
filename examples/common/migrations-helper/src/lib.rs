use sqlorm_core::Pool;
use sqlx::migrate::MigrateError;

#[cfg(feature = "postgres")]
pub async fn run_migrations(pool: &Pool) -> Result<(), MigrateError> {
    sqlx::migrate!("../migrations/postgres").run(pool).await
}

#[cfg(feature = "sqlite")]
pub async fn run_migrations(pool: &Pool) -> Result<(), MigrateError> {
    sqlx::migrate!("../migrations/sqlite").run(pool).await
}
