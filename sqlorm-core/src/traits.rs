use crate::Pool;
use crate::Row;
use crate::TableInfo;
use async_trait::async_trait;

/// Describes a database table and its metadata used by the query builder.
///
/// Implement this trait for your entity types to enable type-safe query building.
/// It provides static metadata such as table name, primary key, and available columns.
pub trait Table {
    /// The table name in the database.
    const TABLE_NAME: &'static str;
    /// The primary key column name.
    const PK: &'static str;
    /// The list of selectable columns for this table.
    const COLUMNS: &'static [&'static str];

    /// Returns a TableInfo instance used by the query builder.
    fn table_info() -> TableInfo;
}

/// Constructs a value from a database row where columns were projected with aliases.
///
/// Implementations should read values from row using the composed alias+column format
/// produced by the query builder's projections.
pub trait FromAliasedRow {
    /// Builds `Self` from an aliased row.
    fn from_aliased_row(row: &Row) -> sqlx::Result<Self>
    where
        Self: Sized + Default;
}

/// Executes a built query and returns typed results.
///
/// This trait is implemented for the query builder type, allowing you to fetch typed
/// rows directly into your domain structs that implement `sqlx::FromRow`.
///
/// # Examples
///
/// PostgreSQL
///
/// ```no_run
/// use sqlorm_core::{qb::{QB, Column}, TableInfo, Executor, Pool};
/// use std::marker::PhantomData;
///
/// # async fn run(pool: &Pool) -> sqlx::Result<()> {
/// let base = TableInfo { name: "users", alias: "u".to_string(), columns: vec!["id", "name"] };
/// let qb1 = QB::<()>::new(base)
///     .select::<(i32, String)>(vec!["id", "name"]) 
///     .filter(Column::<i32> { name: "id", table_alias: "u", _marker: PhantomData }.eq(1));
/// let one: (i32, String) = qb1.fetch_one_as(pool).await?;
/// let qb2 = QB::<()>::new(TableInfo { name: "users", alias: "u".to_string(), columns: vec!["id", "name"] })
///     .select::<(i32, String)>(vec!["id", "name"]) 
///     .filter(Column::<i32> { name: "id", table_alias: "u", _marker: PhantomData }.gt(0));
/// let many: Vec<(i32, String)> = qb2.fetch_all_as(pool).await?;
/// # Ok(())
/// # }
/// ```
///
/// SQLite
///
/// ```no_run
/// use sqlorm_core::{qb::{QB, Column}, TableInfo, Executor, Pool};
/// use std::marker::PhantomData;
///
/// # async fn run(pool: &Pool) -> sqlx::Result<()> {
/// let base = TableInfo { name: "users", alias: "u".to_string(), columns: vec!["id", "name"] };
/// let qb1 = QB::<()>::new(base)
///     .select::<(i32, String)>(vec!["id", "name"]) 
///     .filter(Column::<i32> { name: "id", table_alias: "u", _marker: PhantomData }.eq(1));
/// let one: (i32, String) = qb1.fetch_one_as(pool).await?;
/// let qb2 = QB::<()>::new(TableInfo { name: "users", alias: "u".to_string(), columns: vec!["id", "name"] })
///     .select::<(i32, String)>(vec!["id", "name"]) 
///     .filter(Column::<i32> { name: "id", table_alias: "u", _marker: PhantomData }.gt(0));
/// let many: Vec<(i32, String)> = qb2.fetch_all_as(pool).await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait Executor<T> {
    /// Executes the query and returns a single row mapped as `T`.
    async fn fetch_one_as(self, pool: &Pool) -> sqlx::Result<T>;
    /// Executes the query and returns all rows mapped as `T`.
    async fn fetch_all_as(self, pool: &Pool) -> sqlx::Result<Vec<T>>;
}
