use sqlx::Postgres;
use std::fmt::Debug;

pub trait BindValue:
    for<'q> sqlx::Encode<'q, Postgres> + Debug + sqlx::Type<Postgres> + Send + Sync + 'static
{
}

impl<T> BindValue for T where
    T: for<'q> sqlx::Encode<'q, Postgres> + Debug + sqlx::Type<Postgres> + Send + Sync + 'static
{
}
