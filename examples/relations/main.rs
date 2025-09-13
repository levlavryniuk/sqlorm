//! # Relations Example
//!
//! This example demonstrates handling of database relationships,
//! including eager and lazy loading.
//!
//! Run with: `cargo run --example relations --features="postgres"`

use sqlorm::prelude::*;
use sqlorm::sqlx::Executor;
use uuid::Uuid;

/// Represents a user in the database.
/// A user can have multiple posts.
#[table(name = "users")]
#[derive(Debug, Clone, Default)]
pub struct User {
    #[sql(pk)]
    #[sql(relation(has_many -> Post, relation = "posts", on = user_id))]
    pub id: i64,
    #[sql(unique)]
    pub username: String,
}

impl User {
    pub fn new(username: &str) -> Self {
        Self {
            username: username.to_string(),
            ..Default::default()
        }
    }
}

#[table(name = "posts")]
#[derive(Debug, Clone, Default)]
pub struct Post {
    #[sql(pk)]
    pub id: i64,
    #[sql(relation(belongs_to -> User, relation = "user", on = id))]
    pub user_id: i64,
    pub title: String,
}

impl Post {
    pub fn new(user_id: i64, title: &str) -> Self {
        Self {
            user_id,
            title: title.to_string(),
            ..Default::default()
        }
    }
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
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_test_db().await;

    // Create tables
    pool.execute(
        r#"
        DROP TABLE IF EXISTS "posts";
        DROP TABLE IF EXISTS "users";
        CREATE TABLE "users" (
            id BIGSERIAL PRIMARY KEY,
            username VARCHAR NOT NULL UNIQUE
        );
        CREATE TABLE "posts" (
            id BIGSERIAL PRIMARY KEY,
            user_id BIGINT NOT NULL REFERENCES "users"(id),
            title VARCHAR NOT NULL
        );
    "#,
    )
    .await?;

    println!("Created tables `users` and `posts`");

    let user = User::new("john_doe").save(&pool).await?;
    let post1 = Post::new(user.id, "First Post").save(&pool).await?;
    let post2 = Post::new(user.id, "Second Post").save(&pool).await?;

    println!(
        "
Created user: {:?}",
        user
    );
    println!("Created post: {:?}", post1);
    println!("Created post: {:?}", post2);

    // --- Lazy Loading ---
    println!(
        "
--- Lazy Loading ---"
    );
    let posts = user.posts(&pool).await?;
    println!(
        "Lazy loaded posts for user '{}': {:?}",
        user.username, posts
    );

    let post_owner = post1.user(&pool).await?.expect("User should exist");
    println!(
        "Lazy loaded owner for post '{}': {:?}",
        post1.title, post_owner
    );

    // --- Eager Loading ---
    println!(
        "
--- Eager Loading ---"
    );
    let loaded_user = User::query()
        .filter(User::ID.eq(user.id))
        .with_posts()
        .fetch_one(&pool)
        .await?;

    println!("Eager loaded user with posts: {:?}", loaded_user);
    let loaded_posts = loaded_user.posts.expect("Posts should exist");
    println!("Eager loaded posts: {:?}", loaded_posts);

    let loaded_post = Post::query()
        .filter(Post::ID.eq(post1.id))
        .with_user()
        .fetch_one(&pool)
        .await?;

    println!("Eager loaded post with user: {:?}", loaded_post);
    let loaded_user = loaded_post.user.expect("User should exist");
    println!("Eager loaded user: {:?}", loaded_user);
    println!("Example completed successfully!");

    Ok(())
}
