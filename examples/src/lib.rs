use sqlorm::Pool;

#[cfg(feature = "postgres")]
use std::env;

#[cfg(feature = "postgres")]
pub async fn setup_test_db() -> Pool {
    let database_url = env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://us:pa@localhost:5432/test".to_string());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL database");

    migrations_helper::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

#[cfg(feature = "postgres")]
pub async fn create_clean_db() -> Pool {
    let pool = setup_test_db().await;

    sqlx::query("TRUNCATE TABLE \"donation\", \"jar\", \"user\" RESTART IDENTITY CASCADE")
        .execute(&pool)
        .await
        .expect("Failed to truncate tables");

    pool
}

#[cfg(feature = "sqlite")]
pub async fn create_clean_db() -> Pool {
    create_memory_db().await
}

#[cfg(feature = "sqlite")]
pub async fn create_memory_db() -> Pool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite database");

    migrations_helper::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("Failed to enable foreign keys");

    pool
}
