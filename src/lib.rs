//! # `Sqlorm`
//!
//! **An ergonomic lightweight SQL framework for comfortable database operations.**
//!
//! ## Overview
//!
//! Sqlorm is a lightweight sql framework based on [sqlx](https://github.com/launchbadge/sqlx).
//!
//! It provides macro-generated code for communicating with database, and it is designed to be performant and easy to use.
//!
//! ## Main features:
//! - **Easy to use**: Sqlorm generates all code for specific struct using `sqlorm::table` macro.
//! - **Type-safe (mostly)**: Since 90% of code is geneated at compile time, all types are checked at compile time.
//! - **Performant**: Sqlorm introduces a minimal overhead over sqlx (thanks to macros).
//!
//! To get start, look at [`sqlorm::table`](https://docs.rs/sqlorm/latest/sqlorm/attr.table.html)
//!
//! ## Todo list:
//! - [x] Generic, type-safe query builder for each entity.
//! - [x] Support for postgres.
//! - [x] Support for sqlite.
//! - [x] Support for most common relations (belongs-to, has-many, has-one).
//! - [x] Modular system for optional extended functionality (additional ways to query, etc).
//! - [x] Partial update support ( a big one )
//! - [x] Limit, offset support
//! - [x] Improved filtering system. ( OR, AND )
//! - [x] Renaming fields
//! - [x] Soft delete support
//! - [x] Transactions
//! - [ ] Improved update, delete QueryBuilder. (e.g. User::insert())
//! - [ ] Order-by's
//! - [ ] Cross-relations filters
//! - Problems:
//!     When loading relations batch, we need to move all foreign filters from original query to
//!     batch query. For that we need to rebuild qb and executor
//! //! e.g. User::query().with_posts().filter(Post::read_time.gt_(4))
//!
//!
//!
//! ## Usage
//!
//! To use this crate you must activate **one** of the following features (else the crate wont compile):
//! - `postgres`
//! - `sqlite`

#![cfg(any(feature = "postgres", feature = "sqlite"))]

pub use hashbrown::HashMap;
pub use sqlorm_core::*;
pub use sqlorm_core::{Connection, Driver, GenericExecutor, Pool, Row};
pub use sqlorm_macros::Entity;
pub use sqlorm_macros::table;

pub mod prelude {
    pub use async_trait::async_trait;
    pub use sqlorm_core::*;
    pub use sqlorm_macros::*;
    pub use sqlx;
}
