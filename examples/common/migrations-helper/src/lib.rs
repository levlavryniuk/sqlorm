use sqlorm_core::Pool;
use sqlx::migrate::MigrateError;

/// Runs the migrations located in the ../migrations directory.
/// This function uses the sqlx::migrate! macro to embed the migration files
/// at compile time and run them against the provided database pool.
pub async fn run_migrations(pool: &Pool) -> Result<(), MigrateError> {
    sqlx::migrate!("../migrations").run(pool).await
}
