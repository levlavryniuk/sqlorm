use crate::qb::{bind::BindValue, condition::Condition};
use std::marker::PhantomData;

/// Represents a database column in a type-safe way.
///
/// `Column<T>` is a lightweight wrapper around a column name (`&'static str`)
/// with a phantom type parameter `T` that indicates the type of values
/// that can be bound to conditions involving this column.
///
/// This allows you to write type-safe query conditions such as:
///
/// ```ignore
/// use sqlorm_core::qb::{Column, Condition};
/// use std::marker::PhantomData;
///
/// static ID: Column<i32> = Column { name: "id", table_alias: "user__", _marker: PhantomData };
/// let cond: Condition = ID.eq(42);
/// assert_eq!(cond.sql, "user__.id = ?");
/// ```
pub struct Column<T> {
    /// The column name as it appears in SQL.
    pub name: &'static str,

    /// The column name with table alias, as it appears in SQL.
    /// example: `__user.id`
    pub aliased_name: &'static str,

    /// The table alias to use when generating SQL conditions.
    pub table_alias: &'static str,

    /// Marker to carry the type information for the column.
    pub _marker: PhantomData<T>,
}
impl<T> AsRef<str> for Column<T> {
    fn as_ref(&self) -> &str {
        self.name
    }
}

impl<T> Copy for Column<T> {}
impl<T> Clone for Column<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Column<T>
where
    T: BindValue + Clone,
{
    /// Get the fully qualified column name (with table alias)
    fn qualified_name(&self) -> String {
        format!("{}.{}", self.table_alias, self.name)
    }

    /// Create a condition: `column = ?`
    pub fn eq(self, val: T) -> Condition {
        Condition::new(format!("{} = ?", self.qualified_name()), val)
    }

    /// Create a condition: `column <> ?`
    pub fn ne(self, val: T) -> Condition {
        Condition::new(format!("{} <> ?", self.qualified_name()), val)
    }

    /// Create a condition: `column > ?`
    pub fn gt(self, val: T) -> Condition {
        Condition::new(format!("{} > ?", self.qualified_name()), val)
    }

    /// Create a condition: `column >= ?`
    pub fn ge(self, val: T) -> Condition {
        Condition::new(format!("{} >= ?", self.qualified_name()), val)
    }

    /// Create a condition: `column < ?`
    pub fn lt(self, val: T) -> Condition {
        Condition::new(format!("{} < ?", self.qualified_name()), val)
    }

    /// Create a condition: `column <= ?`
    pub fn le(self, val: T) -> Condition {
        Condition::new(format!("{} <= ?", self.qualified_name()), val)
    }

    /// Create a condition: `column LIKE ?`
    pub fn like(self, val: T) -> Condition {
        Condition::new(format!("{} LIKE ?", self.qualified_name()), val)
    }

    /// Create a condition: `column IN (?, ?, ...)`
    ///
    /// The number of placeholders matches the number of values provided.
    ///
    /// Panics if `vals` is empty
    pub fn in_(self, vals: Vec<T>) -> Condition {
        if vals.is_empty() {
            panic!(
                "Cannot create IN condition with empty value list. At least one value must be specified."
            );
        }
        let placeholders: Vec<String> = (0..vals.len()).map(|_| "?".to_string()).collect();
        let sql = format!("{} IN ({})", self.qualified_name(), placeholders.join(", "));
        Condition::multi(sql, vals)
    }

    /// Create a condition: `column NOT IN (?, ?, ...)`
    ///
    /// Panics if `vals` is empty
    pub fn not_in(self, vals: Vec<T>) -> Condition {
        if vals.is_empty() {
            panic!(
                "Cannot create NOT IN condition with empty value list. At least one value must be specified."
            );
        }
        let placeholders: Vec<String> = (0..vals.len()).map(|_| "?".to_string()).collect();
        let sql = format!(
            "{} NOT IN ({})",
            self.qualified_name(),
            placeholders.join(", ")
        );
        Condition::multi(sql, vals)
    }

    /// Create a condition: `column IS NULL`
    pub fn is_null(self) -> Condition {
        Condition::none(format!("{} IS NULL", self.qualified_name()))
    }

    /// Create a condition: `column IS NOT NULL`
    pub fn is_not_null(self) -> Condition {
        Condition::none(format!("{} IS NOT NULL", self.qualified_name()))
    }

    /// Create a condition: `column BETWEEN ? AND ?`
    pub fn between(self, start: T, end: T) -> Condition {
        let sql = format!("{} BETWEEN ? AND ?", self.qualified_name());
        Condition::multi(sql, vec![start, end])
    }

    /// Create a condition: `column NOT BETWEEN ? AND ?`
    pub fn not_between(self, start: T, end: T) -> Condition {
        let sql = format!("{} NOT BETWEEN ? AND ?", self.qualified_name());
        Condition::multi(sql, vec![start, end])
    }
}
