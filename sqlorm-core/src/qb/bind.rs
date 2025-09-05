use crate::driver::Driver;
use std::fmt::Debug;

pub trait BindValue:
    for<'q> sqlx::Encode<'q, Driver> + Debug + sqlx::Type<Driver> + Send + Sync + 'static
{
}

impl<T> BindValue for T where
    T: for<'q> sqlx::Encode<'q, Driver> + Debug + sqlx::Type<Driver> + Send + Sync + 'static
{
}
