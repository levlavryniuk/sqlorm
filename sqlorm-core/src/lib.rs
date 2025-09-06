pub mod qb;
use sqlx::FromRow;

pub use crate::qb::TableInfo;
pub use async_trait::async_trait;
pub use qb::Column;
pub use qb::Condition;
pub use qb::*;
mod traits;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
pub use driver::{Connection, Driver, Pool, Row};

pub use traits::Executor;
pub use traits::FromAliasedRow;
pub use traits::Table;

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
    #[cfg(all(feature = "postgres", feature = "sqlite"))]
    compile_error!(
        "only one database driver can be set – please enable either 'postgres' or 'sqlite' feature, not both"
    );

    #[cfg(feature = "postgres")]
    /// Sqlorm Database Driver
    pub type Driver = sqlx::Postgres;

    #[cfg(feature = "postgres")]
    /// Sqlorm Database Pool
    pub type Pool = sqlx::PgPool;

    #[cfg(feature = "postgres")]
    /// Sqlorm Database Connection
    pub type Connection = sqlx::PgConnection;

    #[cfg(feature = "postgres")]
    /// Sqlorm Database Row
    pub type Row = sqlx::postgres::PgRow;

    #[cfg(feature = "sqlite")]
    /// Sqlorm Database Driver
    pub type Driver = sqlx::Sqlite;

    #[cfg(feature = "sqlite")]
    /// Sqlorm Database Pool
    pub type Pool = sqlx::SqlitePool;

    #[cfg(feature = "sqlite")]
    /// Sqlorm Database Connection
    pub type Connection = sqlx::SqliteConnection;

    #[cfg(feature = "sqlite")]
    /// Sqlorm Database Row
    pub type Row = sqlx::sqlite::SqliteRow;
}

#[doc(hidden)]
pub use sqlx;
