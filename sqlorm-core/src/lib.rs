pub mod qb;
use sqlx::FromRow;

pub use crate::qb::TableInfo;
pub use async_trait::async_trait;
pub use qb::Column;
pub use qb::Condition;
pub use qb::*;

pub trait Table {
    const TABLE_NAME: &'static str;
    const PK: &'static str;
    const COLUMNS: &'static [&'static str];

    fn table_info() -> TableInfo;
}

pub trait FromAliasedRow {
    fn from_aliased_row(row: &Row) -> sqlx::Result<Self>
    where
        Self: Sized + Default;
}
pub use driver::{Connection, Driver, Pool, Row};

#[async_trait]
pub trait Executor<T> {
    async fn fetch_one_as(self, pool: &Pool) -> sqlx::Result<T>;
    async fn fetch_all_as(self, pool: &Pool) -> sqlx::Result<Vec<T>>;
}

#[async_trait]
impl<T> Executor<T> for QB<T>
where
    T: for<'r> FromRow<'r, Row> + Send + Unpin + std::fmt::Debug,
{
    async fn fetch_one_as(mut self, pool: &Pool) -> sqlx::Result<T> {
        self.eager.clear();
        self.batch.clear();
        let row = self.build_query().build().fetch_one(pool).await?;
        T::from_row(&row)
    }

    async fn fetch_all_as(self, pool: &Pool) -> sqlx::Result<Vec<T>> {
        let rows = self.build_query().build().fetch_all(pool).await?;
        rows.iter().map(T::from_row).collect()
    }
}

#[macro_export]
macro_rules! debug_q {
    ($q:expr) => {{
        use sqlx::Execute;
        let sql = sqlx::Execute::sql(&$q);
        ::std::dbg!(sql);
    }};
}

pub mod driver {
    #[cfg(any(
        all(feature = "postgres", any(feature = "mysql", feature = "sqlite")),
        all(feature = "mysql", any(feature = "postgres", feature = "sqlite")),
        all(feature = "sqlite", any(feature = "postgres", feature = "mysql")),
    ))]
    compile_error!(
        "only one database driver can be set – please use multiple binaries using different atmosphere features if you need more than one database"
    );

    #[cfg(all(feature = "postgres", not(any(feature = "mysql", feature = "sqlite"))))]
    /// Sqlorm Database Driver
    pub type Driver = sqlx::Postgres;

    #[cfg(all(feature = "postgres", not(any(feature = "mysql", feature = "sqlite"))))]
    /// Sqlorm Database Pool
    pub type Pool = sqlx::PgPool;

    #[cfg(all(feature = "postgres", not(any(feature = "mysql", feature = "sqlite"))))]
    /// Sqlorm Database Connection
    pub type Connection = sqlx::PgConnection;

    #[cfg(all(feature = "postgres", not(any(feature = "mysql", feature = "sqlite"))))]
    /// Sqlorm Database Row
    pub type Row = sqlx::postgres::PgRow;

    #[cfg(all(feature = "mysql", not(any(feature = "postgres", feature = "sqlite"))))]
    /// Sqlorm Database Driver
    pub type Driver = sqlx::MySql;

    #[cfg(all(feature = "mysql", not(any(feature = "postgres", feature = "sqlite"))))]
    /// Sqlorm Database Pool
    pub type Pool = sqlx::MySqlPool;

    #[cfg(all(feature = "mysql", not(any(feature = "postgres", feature = "sqlite"))))]
    /// Sqlorm Database Connection
    pub type Connection = sqlx::MySqlConnection;

    #[cfg(all(feature = "mysql", not(any(feature = "postgres", feature = "sqlite"))))]
    /// Sqlorm Database Row
    pub type Row = sqlx::mysql::MySqlRow;

    #[cfg(all(feature = "sqlite", not(any(feature = "postgres", feature = "mysql"))))]
    /// Sqlorm Database Driver
    pub type Driver = sqlx::Sqlite;

    #[cfg(all(feature = "sqlite", not(any(feature = "postgres", feature = "mysql"))))]
    /// Sqlorm Database Pool
    pub type Pool = sqlx::SqlitePool;

    #[cfg(all(feature = "sqlite", not(any(feature = "postgres", feature = "mysql"))))]
    /// Sqlorm Database Connection
    pub type Connection = sqlx::SqliteConnection;

    #[cfg(all(feature = "sqlite", not(any(feature = "postgres", feature = "mysql"))))]
    /// Sqlorm Database Row
    pub type Row = sqlx::sqlite::SqliteRow;
}

// Re-export driver types for use in macros
pub use driver::*;

#[doc(hidden)]
pub use sqlx;
