//! # CRUD Operations Example
//!
//! This example demonstrates comprehensive CRUD operations using SQLOrm.
//! Run with: `cargo run --example crud --features="postgres uuid"`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlorm::GenericExecutor;
use sqlorm::prelude::*;
use sqlorm::sqlx::Executor as SqlxExecutor;
use sqlorm::table;
use std::env;
use uuid::Uuid;

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
    pub bio: Option<String>,
    #[sql(timestamp = "created_at")]
    pub created_at: DateTime<Utc>,
    #[sql(timestamp = "updated_at")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[table(name = "jar")]
pub struct Jar {
    #[sql(pk)]
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub goal: Option<f64>,
    #[sql(relation(belongs_to -> User, relation = "owner", on = id))]
    pub owner_id: i64,
    #[sql(skip)]
    #[sqlx(skip)]
    pub owner: Option<User>,
    #[sql(timestamp = "created_at")]
    pub created_at: DateTime<Utc>,
    #[sql(timestamp = "updated_at")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[table(name = "donation")]
pub struct Donation {
    #[sql(pk)]
    pub id: Uuid,
    pub amount: f64,
    pub message: Option<String>,
    #[sql(relation(belongs_to -> Jar, relation = "jar", on = id))]
    pub jar_id: i64,
    #[sql(relation(belongs_to -> User, relation = "donor", on = id))]
    pub donor_id: i64,
    #[sql(skip)]
    #[sqlx(skip)]
    pub jar: Option<Jar>,
    #[sql(skip)]
    #[sqlx(skip)]
    pub donor: Option<User>,
    #[sql(timestamp = "created_at")]
    pub created_at: DateTime<Utc>,
    #[sql(timestamp = "updated_at")]
    pub updated_at: DateTime<Utc>,
}

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
async fn setup_database(pool: &sqlorm::Pool) -> Result<(), sqlorm::sqlx::Error> {
    // Create tables with proper schema
    sqlorm::sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS "user" (
            id BIGSERIAL PRIMARY KEY,
            email VARCHAR NOT NULL UNIQUE,
            username VARCHAR NOT NULL,
            first_name VARCHAR NOT NULL,
            last_name VARCHAR NOT NULL,
            bio TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#,
    )
    .execute(pool)
    .await?;

    sqlorm::sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS "jar" (
            id BIGSERIAL PRIMARY KEY,
            title VARCHAR NOT NULL,
            description TEXT,
            goal DOUBLE PRECISION,
            owner_id BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#,
    )
    .execute(pool)
    .await?;

    sqlorm::sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS "donation" (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            amount DOUBLE PRECISION NOT NULL,
            message TEXT,
            jar_id BIGINT NOT NULL REFERENCES "jar"(id) ON DELETE CASCADE,
            donor_id BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_test_db().await;

    setup_database(&pool).await?;

    println!("\\n=== CREATE Operations ===");

    // Create users
    let alice = User {
        email: "alice@example.com".to_string(),
        username: "alice".to_string(),
        first_name: "Alice".to_string(),
        last_name: "Smith".to_string(),
        bio: Some("Loves coding and coffee".to_string()),
        ..Default::default()
    }
    .save(&pool)
    .await?;
    println!("✅ Created user Alice: ID={}", alice.id);

    let bob = User {
        email: "bob@example.com".to_string(),
        username: "bob".to_string(),
        first_name: "Bob".to_string(),
        last_name: "Johnson".to_string(),
        ..Default::default()
    }
    .save(&pool)
    .await?;
    println!("✅ Created user Bob: ID={}", bob.id);

    // Create jars
    let alice_jar = Jar {
        title: "Alice's Coffee Fund".to_string(),
        description: Some("Help Alice buy better coffee for coding sessions".to_string()),
        goal: Some(100.0),
        owner_id: alice.id,
        ..Default::default()
    }
    .save(&pool)
    .await?;
    println!("✅ Created jar for Alice: ID={}", alice_jar.id);

    // Create donations with UUID
    let donation1 = Donation {
        amount: 25.0,
        message: Some("Great initiative Alice!".to_string()),
        jar_id: alice_jar.id,
        donor_id: bob.id,
        ..Default::default()
    }
    .save(&pool)
    .await?;
    println!("✅ Created donation: ID={}", donation1.id);

    println!("\\n=== READ Operations ===");

    // Find by ID
    let found_alice = User::find_by_id(&pool, alice.id)
        .await?
        .expect("Alice should exist");
    println!("🔍 Found Alice by ID: {}", found_alice.username);

    // Find by unique field
    let found_by_email = User::find_by_email(&pool, "bob@example.com".to_string())
        .await?
        .expect("Bob should exist");
    println!("🔍 Found Bob by email: {}", found_by_email.username);

    // Query with filters
    let users_with_bio = User::query()
        .filter(User::BIO.is_not_null())
        .fetch_all(&pool)
        .await?;
    println!("🔍 Users with bio: {}", users_with_bio.len());

    println!("\\n=== UPDATE Operations ===");

    // Update user bio
    let mut updated_alice = found_alice;
    updated_alice.bio = Some("Senior Rust developer who loves good coffee".to_string());
    let updated_alice = updated_alice.save(&pool).await?;
    println!("✏️  Updated Alice's bio");

    println!("\\n=== SELECT Operations ===");

    // Select specific fields
    let user_summaries: Vec<(String, String, Option<String>)> = User::query()
        .select(vec![
            User::USERNAME.as_ref(),
            User::EMAIL.as_ref(),
            User::BIO.as_ref(),
        ])
        .fetch_all_as(&pool)
        .await?;

    println!("📋 User summaries:");
    for (username, email, bio) in user_summaries {
        println!(
            "  - {}: {} ({})",
            username,
            email,
            bio.unwrap_or_else(|| "No bio".to_string())
        );
    }

    let total_users = User::query().fetch_all(&pool).await?.len();
    let total_donations = Donation::query().fetch_all(&pool).await?.len();

    println!("\\n📊 Database Summary:");
    println!("   Users: {}", total_users);
    println!("   Donations: {}", total_donations);

    println!("\\n🎉 CRUD example completed successfully!");

    Ok(())
}
