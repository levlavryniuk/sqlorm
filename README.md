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
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls"] }
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

    #[sql(timestamp = "created_at")]
    pub created_at: DateTime<Utc>,

    #[sql(timestamp = "updated_at")]
    pub updated_at: DateTime<Utc>,
}
```

### Basic CRUD Operations

```rust
// Create
let mut user = User {
    email: "user@example.com".to_string(),
    name: "John Doe".to_string(),
    ..Default::default()
};
let saved_user = user.save(&pool).await?;

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

### Using Just (Recommended)

```bash
# Quick development workflow
just dev                    # Compile check + run core tests
just examples              # Test all examples (PostgreSQL + SQLite)
just examples-help         # Show all available example commands

# Test specific features
just examples-postgres     # Test PostgreSQL examples only
just examples-sqlite       # Test SQLite examples only
just examples-crud         # Test CRUD operations
just examples-filters      # Test query filters
just examples-relations    # Test relationships
just examples-select       # Test custom selects

# Full test suite
just ci                    # Complete CI test suite
just all                   # All tests including examples
```

### Using Cargo Directly

```bash
# Build with PostgreSQL driver
cargo build --features postgres

# Build with SQLite driver
cargo build --features sqlite

# Run tests
cargo test --workspace --features postgres
cargo test --workspace --features sqlite

# Check all features
cargo check --features postgres
cargo check --features sqlite
```

## Examples

This repository includes comprehensive examples demonstrating all SQLOrm features for both database drivers:

### Quick Start

```bash
# Test all examples for both databases
just examples

# Development workflow
just dev                   # Quick compile check + core tests
just examples-help         # Show all available commands

# Test specific databases
just examples-postgres     # PostgreSQL only
just examples-sqlite       # SQLite only

# Test specific functionality
just examples-crud         # CRUD operations
just examples-filters      # Query building
just examples-relations    # Relationships
```

### Running PostgreSQL Examples

#### With Just (Recommended)

```bash
# Run all PostgreSQL examples
just examples-postgres

# Run specific test suites
just examples-postgres-suite crud
just examples-postgres-suite filters
just examples-postgres-suite relations
just examples-postgres-suite select

# Test with custom database URL
just examples-postgres-custom 'postgresql://user:pass@localhost/testdb'

# Verbose output for debugging
just examples-postgres-verbose
```

#### With Cargo Directly

```bash
# Set up a PostgreSQL test database (optional)
export TEST_DATABASE_URL="postgresql://user:password@localhost/test_db"

# Run PostgreSQL example tests
cargo test -p sqlorm-postgres-example --features postgres

# Run specific test suites
cargo test -p sqlorm-postgres-example --features postgres --test crud
cargo test -p sqlorm-postgres-example --features postgres --test filters
cargo test -p sqlorm-postgres-example --features postgres --test relations
cargo test -p sqlorm-postgres-example --features postgres --test select
```

### Running SQLite Examples

#### With Just (Recommended)

```bash
# Run all SQLite examples (no setup required)
just examples-sqlite

# Run specific test suites
just examples-sqlite-suite crud
just examples-sqlite-suite filters
just examples-sqlite-suite relations
just examples-sqlite-suite select

# Verbose output for debugging
just examples-sqlite-verbose
```

#### With Cargo Directly

```bash
# Run SQLite example tests (no setup required)
cargo test -p sqlorm-sqlite-example --features sqlite

# Run specific test suites
cargo test -p sqlorm-sqlite-example --features sqlite --test crud
cargo test -p sqlorm-sqlite-example --features sqlite --test filters
cargo test -p sqlorm-sqlite-example --features sqlite --test relations
cargo test -p sqlorm-sqlite-example --features sqlite --test select
```

## Example Structure

The examples are organized as follows:

- `examples/common/entities/` - Shared entity definitions (User, Jar, Donation)
- `examples/common/migrations/` - Database migrations
- `examples/common/migrations-helper/` - Migration runner utility
- `examples/postgres/` - PostgreSQL-specific example tests
- `examples/sqlite/` - SQLite-specific example tests

Each example crate includes tests for:

- **CRUD Operations** (`crud.rs`) - Create, read, update, delete operations
- **Query Filters** (`filters.rs`) - All query builder filter operations
- **Relationships** (`relations.rs`) - Lazy and eager loading of related entities
- **Custom Selects** (`select.rs`) - Column projection and custom result mapping

## Features Demonstrated

### Entity Generation

- Primary keys with `#[sql(pk)]`
- Unique fields with `#[sql(unique)]` (generates `find_by_*` methods)
- Automatic timestamps with `#[sql(timestamp = "created_at|updated_at")]`
- Table name customization with `#[table_name(name = "...")]`

### Query Builder

- Comparison operators: `eq`, `ne`, `gt`, `ge`, `lt`, `le`
- Pattern matching: `like`
- Set operations: `in_`, `not_in`
- Null checks: `is_null`, `is_not_null`
- Range queries: `between`, `not_between`
- Multiple filters with AND logic

### Relationships

- `belongs_to` - Load parent entities
- `has_many` - Load child entity collections
- `has_one` - Load single related entities
- Lazy loading via instance methods
- Eager loading via query builder `.with_*()` methods

### Custom Queries

- Column selection with `.select()`
- Result mapping to tuples and custom types
- Combining filters with custom selects

## Documentation

For detailed API documentation, see the individual crate docs:

- [sqlorm](./src/lib.rs) - Main crate documentation
- [sqlorm-core](./sqlorm-core/src/lib.rs) - Core traits and query builder
- [sqlorm-macros](./sqlorm-macros/src/lib.rs) - Entity macro documentation

## Development Guidelines

- Use `expect()` instead of `unwrap()` for error handling
- Document public APIs with rustdoc (`///`)
- No inline comments (`//`) - use rustdoc for documentation
- Follow clippy pedantic lints
- Test with both PostgreSQL and SQLite drivers

## License

[Add your license here]
