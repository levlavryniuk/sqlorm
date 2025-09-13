# SQLOrm

A modular, type-safe SQL ORM for Rust with procedural macro-based entity generation and a powerful query builder.

## Architecture

SQLOrm is split into three main crates:

- **`sqlorm`**: The main crate that users depend on
- **`sqlorm-core`**: Core traits and query builder implementation
- **`sqlorm-macros`**: Procedural macros for entity generation

## Database Support

SQLOrm supports two database backends via feature flags:

- **PostgreSQL** (`postgres` feature)
- **SQLite** (`sqlite` feature)

⚠️ **Important**: Only one database driver can be active at a time.

## Quick Start

Add SQLOrm to your `Cargo.toml`:

```toml
[dependencies]
sqlorm = { version = "0.1", features = ["postgres"] }  # or "sqlite"
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls", "chrono"] }
tokio = { version = "1.0", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
```

### Define Entities

```rust
use sqlorm::Entity;
use chrono::{DateTime, Utc};

#[derive(Debug, Entity, sqlx::FromRow, Clone)]
#[table_name(name = "users")]
pub struct User {
    #[sql(pk)]
    pub id: i64,

    #[sql(unique)]
    pub email: String,

    pub name: String,

    #[sql(timestamp(created_at, chrono::Utc::now()))]
    pub created_at: DateTime<Utc>,

    #[sql(timestamp(updated_at, chrono::Utc::now()))]
    pub updated_at: DateTime<Utc>,
}
```

### Basic CRUD Operations

```rust
// Create
let user = User {
    email: "user@example.com".to_string(),
    name: "John Doe".to_string(),
    ..Default::default()
};
// returns the inserted `User` record
let user = user.save(&pool).await?;


// Read
let user = User::find_by_id(&pool, 1).await?;
let user = User::find_by_email(&pool, "user@example.com".to_string()).await?;

// Update
user.name = "Jane Doe".to_string();
let updated_user = user.save(&pool).await?;

// Query with filters
let users = User::query()
    .filter(User::NAME.like("%John%".to_string()))
    .fetch_all(&pool)
    .await?;
```

## Development Commands

```bash
cargo test --workspace --features postgres
```

or

```bash
cargo test --workspace --features sqlite
```
