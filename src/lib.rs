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
//! ## Todo list:
//! - [x] Generic, type-safe query builder for each entity.
//! - [x] Support for postgres.
//! - [x] Support for sqlite.
//! - [x] Support for most common relations (belongs-to, has-many, has-one).
//! - [ ] Modular system for optional extended functionality (additional ways to query, etc).
//! - [ ] Smart prelude modules generation.
//!
//!
//!
//! ## Usage
//!
//! To use this crate you must activate **one** of the following features (else the crate wont compile):
//! - `postgres`
//! - `sqlite`

#![cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]

pub use sqlorm_core::*;
pub use sqlorm_core::{Connection, Driver, Executor, Pool, Row};
pub use sqlorm_macros::Entity;
pub use sqlorm_macros::table;
pub use sqlorm_macros::*;
pub mod prelude {
    pub use async_trait::async_trait;
    pub use sqlorm_core::*;
    pub use sqlorm_macros::*;
    pub use sqlx;
}
