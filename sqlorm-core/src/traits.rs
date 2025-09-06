use crate::Pool;
use crate::Row;
use crate::TableInfo;
use async_trait::async_trait;

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

#[async_trait]
pub trait Executor<T> {
    async fn fetch_one_as(self, pool: &Pool) -> sqlx::Result<T>;
    async fn fetch_all_as(self, pool: &Pool) -> sqlx::Result<Vec<T>>;
}
