use sqlorm_core::Pool;
use tempfile::NamedTempFile;

/// Creates a test SQLite database pool using a temporary file.
/// 
/// This function:
/// 1. Creates a temporary SQLite database file  
/// 2. Creates a connection pool
/// 3. Runs migrations automatically
/// 4. Returns the pool ready for testing
/// 
/// Each call creates a completely isolated database, ensuring
/// tests don't interfere with each other.
pub async fn create_clean_db() -> Pool {
    // Create a temporary database file
    let temp_file = NamedTempFile::new()
        .expect("Failed to create temporary file");
    
    let database_url = format!("sqlite://{}", temp_file.path().to_str().expect("Invalid path"));

    let pool = sqlx::SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to SQLite database");

    // Run migrations
    migrations_helper::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    // Enable foreign key constraints in SQLite
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("Failed to enable foreign keys");

    pool
}

/// Alternative setup function that uses an in-memory database.
/// Useful for very fast tests that don't need persistence.
pub async fn create_memory_db() -> Pool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite database");

    // Run migrations
    migrations_helper::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    // Enable foreign key constraints
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("Failed to enable foreign keys");

    pool
}
