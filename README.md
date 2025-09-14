<div align="center">
  <img src="https://github.com/levlavryniuk/sqlorm/blob/master/logo.png" 
       alt="logo" 
       width="200" />
</div>

# sqlorm

**An ergonomic, lightweight SQL ORM for Rust with type-safe query building and powerful entity relationships.**

[![Crates.io](https://img.shields.io/crates/v/sqlorm.svg)](https://crates.io/crates/sqlorm)
[![Documentation](https://docs.rs/sqlorm/badge.svg)](https://docs.rs/sqlorm)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://github.com/levlavryniuk/sqlorm/blob/main/LICENSE)

Sqlorm is a modern ORM built on top of [sqlx](https://github.com/launchbadge/sqlx) that provides compile-time safety, powerful macro-generated APIs, and an intuitive query builder. It's designed for developers who want the performance of sqlx with the convenience of an ORM.

## ✨ Key Features

- ** Type-Safe **: All queries are checked at compile-time
- ** Zero-Cost Abstraction**: Minimal overhead over raw sqlx
- ** Macro-Powered**: Rich APIs generated from simple struct definitions
- ** Relationships**: Support for `belongs_to` and `has_many` relations with eager/lazy loading
- ** Automatic Timestamps**: Built-in `created_at`/`updated_at` handling
- ** Multi-Database**: PostgreSQL and SQLite support
- ** Powerful Querying**: Fluent query builder with comprehensive filtering

## 🚀 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sqlorm = { version = "0.4", features = ["postgres", "uuid" ] }
# Or for SQLite:
# sqlorm = { version = "0.4", features = ["sqlite", "uuid" ] }

sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls"] }
tokio = { version = "1.0", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
serde = { version = "1.0", features = ["derive"] }
```

### Database Support

Choose **one** feature:

- `postgres` - PostgreSQL support
- `sqlite` - SQLite support

Optional features:

- `uuid` - UUID support
- `extra-traits` - Additional trait derivations

### Your First Entity

```rust
use sqlorm::prelude::*;
use chrono::{DateTime, Utc};

#[table(name = "users")]
#[derive(Debug, Clone, Default)]
pub struct User {
    #[sql(pk)]
    pub id: i64,

    #[sql(unique)]
    pub email: String,

    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub bio: Option<String>,

    #[sql(timestamp(created_at, chrono::Utc::now()))]
    pub created_at: DateTime<Utc>,

    #[sql(timestamp(updated_at, chrono::Utc::now()))]
    pub updated_at: DateTime<Utc>,
}
```

## 📖 Usage Examples

### Basic CRUD Operations

```rust
use sqlorm::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = Pool::connect("postgres://user:pass@localhost/db").await?;

    // CREATE - Insert a new user
    let user = User {
        email: "alice@example.com".to_string(),
        username: "alice".to_string(),
        first_name: "Alice".to_string(),
        last_name: "Smith".to_string(),
        bio: Some("Rust developer".to_string()),
        ..Default::default()
    }
    .save(&pool)  // Returns the inserted user with generated ID and timestamp fields
    .await?;

    println!("Created user with ID: {}", user.id);

    // READ - Find by primary key
    let found_user = User::find_by_id(&pool, user.id)
        .await?
        .expect("User should exist");

    // READ - Find by unique field. Note, this requires `extra-traits` feature
    let found_by_email = User::find_by_email(&pool, "alice@example.com".to_string())
        .await?
        .expect("User should exist");

    // UPDATE - Modify and save
    let mut updated_user = found_user;
    updated_user.bio = Some("Senior Rust developer".to_string());
    let updated_user = updated_user.save(&pool).await?;  // updated_at auto-updated

    // DELETE - Using raw sqlx for now
    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&pool)
        .await?;

    Ok(())
}
```

### Advanced Querying

SQLOrm provides a powerful, type-safe query builder:

```rust
// Simple filtering
let active_users = User::query()
    .filter(User::BIO.is_not_null())
    .fetch_all(&pool)
    .await?;

// Comparison operators
let recent_users = User::query()
    .filter(User::CREATED_AT.gt(chrono::Utc::now() - chrono::Duration::days(30)))
    .filter(User::ID.ge(100))
    .fetch_all(&pool)
    .await?;

// Pattern matching
let rust_developers = User::query()
    .filter(User::BIO.like("%Rust%".to_string()))
    .fetch_all(&pool)
    .await?;

// Multiple conditions
let specific_users = User::query()
    .filter(User::ID.in_(vec![1, 2, 3, 4, 5]))
    .filter(User::EMAIL.ne("admin@example.com".to_string()))
    .fetch_all(&pool)
    .await?;

// Range queries
let mid_range_users = User::query()
    .filter(User::ID.between(10, 50))
    .fetch_one(&pool)  // Get first result
    .await?;

// NULL checks
let users_without_bio = User::query()
    .filter(User::BIO.is_null())
    .fetch_all(&pool)
    .await?;
```

### Selective Field Queries

Optimize your queries by selecting only needed fields:

```rust
// Select specific fields as tuple
let (id, email): (i64, String) = User::query()
    .filter(User::USERNAME.eq("alice".to_string()))
    .select(vec![User::ID.as_ref(), User::EMAIL.as_ref()])
    .fetch_one_as(&pool)
    .await?;

// Select multiple fields
let user_summaries: Vec<(String, String, Option<String>)> = User::query()
    .select(vec![
        User::USERNAME.as_ref(),
        User::EMAIL.as_ref(),
        User::BIO.as_ref(),
    ])
    .fetch_all_as(&pool)
    .await?;
```

### Relationships

Define and work with entity relationships:

```rust
#[table(name = "posts")]
#[derive(Debug, Clone, Default)]
pub struct Post {
    #[sql(pk)]
    pub id: i64,

    pub title: String,
    pub content: String,

    // Foreign key relationship
    #[sql(relation(belongs_to -> User, relation = "author", on = id))]
    pub user_id: i64,

    #[sql(timestamp(created_at, chrono::Utc::now()))]
    pub created_at: DateTime<Utc>,
}

#[table(name = "users")]
#[derive(Debug, Clone, Default)]
pub struct User {
    #[sql(pk)]
    // Define reverse relationship
    #[sql(relation(has_many -> Post, relation = "posts", on = user_id))]
    pub id: i64,

    // ... other fields
}

// Lazy loading - fetch related data when needed
let user = User::find_by_id(&pool, 1).await?.expect("User exists");
let user_posts = user.posts(&pool).await?;  // Separate query

let post = Post::find_by_id(&pool, 1).await?.expect("Post exists");
let author = post.author(&pool).await?.expect("Author exists");

// Eager loading - fetch related data in one query
let user_with_posts = User::query()
    .filter(User::ID.eq(1))
    .with_posts()  // JOIN posts in single query
    .fetch_one(&pool)
    .await?;

let posts = user_with_posts.posts.expect("Posts loaded");

let post_with_author = Post::query()
    .filter(Post::ID.eq(1))
    .with_author()
    .fetch_one(&pool)
    .await?;

let author = post_with_author.author.expect("Author loaded");
```

### Automatic Timestamps

SQLOrm automatically handles timestamp fields:

```rust
#[table]
#[derive(Debug, Clone, Default)]
pub struct Article {
    #[sql(pk)]
    pub id: i64,

    pub title: String,

    // Automatically set on insert
    #[sql(timestamp(created_at, chrono::Utc::now()))]
    pub created_at: DateTime<Utc>,

    // Automatically updated on save
    #[sql(timestamp(updated_at, chrono::Utc::now()))]
    pub updated_at: DateTime<Utc>,

    // Optional soft delete timestamp
    #[sql(timestamp(deleted_at, chrono::Utc::now()))]
    pub deleted_at: Option<DateTime<Utc>>,
}

// Custom timestamp functions
#[sql(timestamp(created_at, get_custom_timestamp()))]
pub created_at: i64,  // Unix timestamp

fn get_custom_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
```

### Multiple Primary Key Types

SQLOrm supports various primary key types:

```rust
use uuid::Uuid;

// Auto-incrementing integer
#[table]
#[derive(Debug, Clone, Default)]
pub struct User {
    #[sql(pk)]
    pub id: i64,  // BIGSERIAL in PostgreSQL
    // ...
}

// UUID primary key
#[table]
#[derive(Debug, Clone, Default)]
pub struct Session {
    #[sql(pk)]
    pub id: Uuid,  // Auto-generated UUID
    // ...
}

// Custom primary key
#[table]
#[derive(Debug, Clone, Default)]
pub struct Setting {
    #[sql(pk)]
    pub key: String,  // String primary key
    pub value: String,
    // ...
}
```

### Working with Options and Different Types

```rust
#[table]
#[derive(Debug, Clone, Default)]
pub struct Product {
    #[sql(pk)]
    pub id: i64,

    pub name: String,
    pub description: Option<String>,    // Nullable text
    pub price: f64,                     // Numeric
    pub is_active: bool,                // Boolean
    pub tags: Option<Vec<String>>,      // JSON array (PostgreSQL)
    pub metadata: Option<serde_json::Value>,  // JSON

    #[sql(timestamp(created_at, chrono::Utc::now()))]
    pub created_at: DateTime<Utc>,
}
```

## 🏗️ Architecture

SQLOrm is built with a modular architecture:

```
┌─────────────────┐
│     sqlorm      │  ← Main crate (user-facing API)
└─────────────────┘
         │
    ┌────┴─────┬────────────────┐
    │          │                │
┌───▼───┐ ┌────▼────┐ ┌─────────▼──┐
│ core  │ │ macros  │ │    sqlx    │
└───────┘ └─────────┘ └────────────┘
```

- **`sqlorm`**: Main crate with public API
- **`sqlorm-core`**: Query builder and core traits
- **`sqlorm-macros`**: Procedural macros for code generation
- **`sqlx`**: Underlying database driver

## 🔧 Generated API Reference

The `#[table]` macro generates extensive APIs for each entity:

### Core Methods

- `save()` - Insert or update (smart detection)
- `insert()` - Force insert
- `update()` - Force update
- `find_by_id()` - Find by primary key
- `find_by_<unique_field>()` - Find by unique fields

### Query Builder

- `query()` - Start query builder
- `filter()` - Add WHERE conditions
- `select()` - Specify columns to fetch
- `fetch_one()` - Get single result
- `fetch_all()` - Get all results
- `fetch_one_as()` - Get result as tuple/custom type
- `fetch_all_as()` - Get results as Vec of tuples/custom type

### Filter Operators

- `eq()` / `ne()` - Equality / Not equal
- `gt()` / `ge()` - Greater than / Greater equal
- `lt()` / `le()` - Less than / Less equal
- `like()` - Pattern matching
- `in_()` / `not_in()` - List membership
- `between()` / `not_between()` - Range queries
- `is_null()` / `is_not_null()` - NULL checks

### Relationships (when defined)

- `<relation_name>()` - Lazy load related entities
- `with_<relation_name>()` - Eager load in query builder

## 📋 Attribute Reference

### Table Attributes

```rust
#[table]                           // Use struct name as table name
#[table(name = "custom_name")]     // Custom table name
```

### Field Attributes

```rust
#[sql(pk)]                                    // Primary key
#[sql(unique)]                                // Unique constraint
#[sql(timestamp(created_at, chrono::Utc::now()))]  // Auto timestamp
#[sql(relation(belongs_to -> Parent, relation = "parent", on = id))]     // Belongs to
#[sql(relation(has_many -> Child, relation = "children", on = parent_id))]  // Has many
```

## 🧪 Testing

```bash
# Test with PostgreSQL
cargo test --features postgres

# Test with SQLite
cargo test --features sqlite

# Run examples
cargo run --example basic --features "postgres uuid chrono"
cargo run --example crud --features "postgres uuid chrono"
cargo run --example relations --features "postgres uuid chrono"
```

## 📚 More Examples

Check the [`examples/`](./examples) directory for complete working examples:

- **[`basic`](./examples/basic/main.rs)**: Simple CRUD operations
- **[`crud`](./examples/crud/main.rs)**: Comprehensive CRUD with multiple entities
- **[`relations`](./examples/relations/main.rs)**: Working with entity relationships

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. Make sure to:

1. Run tests with both database features
2. Follow the existing code style
3. Add tests for new features
4. Update documentation as needed

## 📝 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

Built on the excellent [sqlx](https://github.com/launchbadge/sqlx) crate. Inspired by Rails Active Record and Laravel Eloquent.
