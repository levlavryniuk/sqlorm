//! # Basic SQLOrm Example
//!
//! This example demonstrates the basic usage of SQLOrm with PostgreSQL.
//! Run with: `cargo run --example basic --features="postgres uuid extra-traits"`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlorm::prelude::*;
use sqlorm::sqlx::Executor;
use sqlorm::table;

use uuid::Uuid;

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
// Define a simple User entity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[table(name = "user")]
pub struct User {
    #[sql(pk)]
    pub id: i64,
    #[sql(unique)]
    pub email: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    #[sql(timestamp = "created_at")]
    pub created_at: DateTime<Utc>,
    #[sql(timestamp = "updated_at")]
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(email: &str, username: &str, first_name: &str, last_name: &str) -> Self {
        Self {
            email: email.to_string(),
            username: username.to_string(),
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_test_db().await;

    // Create the users table
    sqlorm::sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS "user" (
            id BIGSERIAL PRIMARY KEY,
            email VARCHAR NOT NULL UNIQUE,
            username VARCHAR NOT NULL UNIQUE,
            first_name VARCHAR NOT NULL,
            last_name VARCHAR NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#,
    )
    .execute(&pool)
    .await?;

    println!("Created users table");

    // Create and save a new user
    let user = User::new("alice@example.com", "alice", "Alice", "Smith")
        .save(&pool)
        .await?;

    println!("Created user: {:?}", user);

    // Find the user by ID
    let found_user = User::find_by_id(&pool, user.id)
        .await?
        .expect("User should exist");

    println!("Found user by ID: {:?}", found_user);

    // Update the user
    let mut updated_user = found_user;
    updated_user.username = "alice_updated".to_string();
    let updated_user = updated_user.save(&pool).await?;

    println!("Updated user: {:?}", updated_user);

    // Query users with filtering
    let users_named_alice = User::query()
        .filter(User::FIRST_NAME.eq("Alice".to_string()))
        .fetch_all(&pool)
        .await?;

    println!("Users named Alice: {:?}", users_named_alice);

    println!("Basic example completed successfully!");

    Ok(())
}
