use std::fmt::Display;

use crate::QB;

#[derive(Debug)]
pub enum Ordering {
    Asc,
    Desc,
}

#[derive(Debug)]
pub struct OrderBySpec {
    pub column: String,
    pub order: Ordering,
}

impl<T> QB<T> {
    /// Applies order by clause to query.
    ///
    /// Example usage:
    /// ```rust ignore
    /// #[table]
    /// struct User {
    ///     pub rating: i32
    ///     pub name: String
    /// }
    /// User::query().order_by(User::RATING.asc()).fetch_many(&pool);
    /// // or
    /// User::query().order_by(User::RATING.desc()).fetch_many(&pool);
    ///
    /// User::query().order_by(User::RATING.desc()).order_by(User::NAME.asc()).fetch_many(&pool);
    /// // turns into: select ... from "user" order by rating desc, name asc
    /// ```
    ///
    pub fn order_by(mut self, stmt: OrderBySpec) -> QB<T> {
        self.order_by.push(stmt);

        self
    }
}
impl Display for Ordering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ordering::Desc => f.write_str("desc"),
            Ordering::Asc => f.write_str("asc"),
        }
    }
}
