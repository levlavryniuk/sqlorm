pub mod entities;

use sqlorm::Pool;
use sqlorm::sqlx::migrate::MigrateError;

#[cfg(feature = "postgres")]
use sqlorm::sqlx::Executor;
#[cfg(feature = "postgres")]
use uuid::Uuid;

#[cfg(feature = "postgres")]
pub async fn run_migrations(pool: &Pool) -> Result<(), MigrateError> {
    sqlorm::sqlx::migrate!("tests/common/migrations/postgres")
        .run(pool)
        .await
}

#[cfg(feature = "sqlite")]
pub async fn run_migrations(pool: &Pool) -> Result<(), MigrateError> {
    sqlorm::sqlx::migrate!("tests/common/migrations/sqlite")
        .run(pool)
        .await
}

#[cfg(feature = "postgres")]
pub async fn create_test_db() -> Pool {
    let base_url = "postgres://test:test@localhost:5432/".to_string();
    let admin_pool = Pool::connect(&base_url).await.unwrap();

    let db_name = format!("test_db_{}", Uuid::new_v4().to_string().replace("-", ""));
    admin_pool
        .execute(format!(r#"CREATE DATABASE "{}""#, db_name).as_str())
        .await
        .expect("Failed to create test database");

    let mut test_db_url = base_url.clone();
    if let Some(idx) = test_db_url.rfind('/') {
        test_db_url.replace_range(idx + 1.., &db_name);
    }

    Pool::connect(&test_db_url)
        .await
        .expect("Failed to connect to test database")
}

#[cfg(feature = "postgres")]
#[allow(dead_code)]
pub async fn setup_test_db() -> Pool {
    use std::env;
    
    let database_url = env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://test:test@localhost:5432/test".to_string());

    let pool = sqlorm::Pool::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL database");

    run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

#[cfg(feature = "postgres")]
pub async fn create_clean_db() -> Pool {
    let pool = create_test_db().await;
    run_migrations(&pool)
        .await
        .expect("Failed to run migrations");
    pool
}

#[cfg(feature = "sqlite")]
pub async fn create_clean_db() -> Pool {
    create_memory_db().await
}

#[cfg(feature = "sqlite")]
pub async fn create_memory_db() -> Pool {
    let pool = sqlorm::Pool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite database");

    run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    sqlorm::sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("Failed to enable foreign keys");

    pool
}
