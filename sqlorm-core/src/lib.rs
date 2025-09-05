pub mod qb;
use sqlx::FromRow;
use sqlx::postgres::PgRow;

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
    fn from_aliased_row(row: &PgRow) -> sqlx::Result<Self>
    where
        Self: Sized + Default;
}

#[async_trait]
pub trait Executor<T> {
    async fn fetch_one_as(self, pool: &sqlx::PgPool) -> sqlx::Result<T>;
    async fn fetch_all_as(self, pool: &sqlx::PgPool) -> sqlx::Result<Vec<T>>;
}

#[async_trait]
impl<T> Executor<T> for QB<T>
where
    T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + std::fmt::Debug,
{
    async fn fetch_one_as(mut self, pool: &sqlx::PgPool) -> sqlx::Result<T> {
        self.eager.clear();
        self.batch.clear();
        let row = self.build_query().build().fetch_one(pool).await?;
        T::from_row(&row)
    }

    async fn fetch_all_as(self, pool: &sqlx::PgPool) -> sqlx::Result<Vec<T>> {
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
    /// Atmosphere Database Driver
    pub type Driver = sqlx::Postgres;

    #[cfg(all(feature = "postgres", not(any(feature = "mysql", feature = "sqlite"))))]
    /// Atmosphere Database Pool
    pub type Pool = sqlx::PgPool;

    #[cfg(all(feature = "mysql", not(any(feature = "postgres", feature = "sqlite"))))]
    /// Atmosphere Database Driver
    pub type Driver = sqlx::MySql;

    #[cfg(all(feature = "mysql", not(any(feature = "postgres", feature = "sqlite"))))]
    /// Atmosphere Database Pool
    pub type Pool = sqlx::MySqlPool;

    #[cfg(all(feature = "sqlite", not(any(feature = "postgres", feature = "mysql"))))]
    /// Atmosphere Database Driver
    pub type Driver = sqlx::Sqlite;

    #[cfg(all(feature = "sqlite", not(any(feature = "postgres", feature = "mysql"))))]
    /// Atmosphere Database Pool
    pub type Pool = sqlx::SqlitePool;
}

#[doc(hidden)]
pub use sqlx;
