use crate::Driver;
use crate::qb::BindValue;
use sqlx::QueryBuilder;

/// Represents a SQL condition fragment with its associated bound values.
///
/// A `Condition` is essentially a piece of SQL (e.g. `"id = $1"`)
/// along with one or more values that should be bound into the query.
/// It is designed to be used with [`sqlx::QueryBuilder`] for dynamic
/// query construction.
pub struct Condition {
    /// The raw SQL fragment (e.g. `"id = $1"`, `"name IN (...)"`).
    pub sql: String,

    /// The values to be bound into the SQL fragment.
    ///
    /// Each value is stored as a boxed [`AnyValue`] trait object,
    /// which allows heterogeneous types to be stored in the same vector.
    pub values: Vec<Box<dyn AnyValue>>,
}

/// Trait representing a value that can be bound into a SQL query.
///
/// This trait abstracts over different types that implement
/// [`BindValue`] and allows them to be stored in a type-erased form
/// (`Box<dyn AnyValue>`).
pub trait AnyValue: Send + Sync {
    /// Bind this value into the given [`QueryBuilder`].
    fn bind(&self, builder: &mut QueryBuilder<'static, Driver>);
}

impl<T> AnyValue for T
where
    T: BindValue + Clone + std::fmt::Debug + 'static,
{
    fn bind(&self, builder: &mut QueryBuilder<'static, Driver>) {
        builder.push_bind(self.clone());
    }
}

impl Condition {
    /// Create a new `Condition` with a single bound value.
    ///
    /// # Example
    /// ```
    /// use macros_core::qb::condition::Condition;
    /// use macros_core::qb::BindValue;
    ///
    /// let cond = Condition::new("id = $1".to_string(), 42);
    /// assert_eq!(cond.sql, "id = $1");
    /// assert_eq!(cond.values.len(), 1);
    /// ```
    pub fn new<T: BindValue + Clone + 'static>(sql: String, val: T) -> Self {
        Self {
            sql,
            values: vec![Box::new(val)],
        }
    }

    /// Create a new `Condition` with multiple bound values.
    ///
    /// Useful for `IN` clauses or other multi-value conditions.
    ///
    /// # Example
    /// ```
    /// use macros_core::qb::condition::Condition;
    /// use macros_core::qb::BindValue;
    ///
    /// let cond = Condition::multi("id IN (...)".to_string(), vec![1, 2, 3]);
    /// assert_eq!(cond.sql, "id IN (...)");
    /// assert_eq!(cond.values.len(), 3);
    /// ```
    pub fn multi<T: BindValue + Clone + 'static>(sql: String, vals: Vec<T>) -> Self {
        Self {
            sql,
            values: vals
                .into_iter()
                .map(|v| Box::new(v) as Box<dyn AnyValue>)
                .collect(),
        }
    }

    /// Create a new `Condition` with no bound values.
    ///
    /// Useful for static SQL fragments that don’t require parameters.
    ///
    /// # Example
    /// ```
    /// use macros_core::qb::condition::Condition;
    ///
    /// let cond = Condition::none("deleted_at IS NULL".to_string());
    /// assert_eq!(cond.sql, "deleted_at IS NULL");
    /// assert!(cond.values.is_empty());
    /// ```
    pub fn none(sql: String) -> Self {
        Self {
            sql,
            values: vec![],
        }
    }
}

impl std::fmt::Debug for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Condition")
            .field("sql", &self.sql)
            .field("values_len", &self.values.len())
            .finish()
    }
}
