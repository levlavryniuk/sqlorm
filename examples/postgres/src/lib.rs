use sqlorm_core::Pool;
use std::env;

/// Creates a test PostgreSQL database pool.
/// 
/// This function:
/// 1. Checks for TEST_DATABASE_URL environment variable
/// 2. Falls back to a default localhost PostgreSQL connection
/// 3. Creates a connection pool
/// 4. Runs migrations automatically
/// 
/// # Environment Variables
/// 
/// - `TEST_DATABASE_URL`: Full PostgreSQL connection string (optional)
/// 
/// # Example
/// 
/// ```bash
/// export TEST_DATABASE_URL="postgresql://user:password@localhost/test_db"
/// cargo test
/// ```
pub async fn setup_test_db() -> Pool {
    let database_url = env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/sqlorm_test".to_string());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL database");

    // Run migrations
    migrations_helper::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Creates a clean test database for each test.
/// This ensures tests don't interfere with each other.
pub async fn create_clean_db() -> Pool {
    let pool = setup_test_db().await;
    
    // Clean up existing data in reverse order due to foreign key constraints
    sqlx::query("DELETE FROM \"donations\"")
        .execute(&pool)
        .await
        .expect("Failed to clean donations table");
        
    sqlx::query("DELETE FROM \"jars\"")
        .execute(&pool)
        .await
        .expect("Failed to clean jars table");
        
    sqlx::query("DELETE FROM \"users\"")
        .execute(&pool)
        .await
        .expect("Failed to clean users table");

    pool
}
