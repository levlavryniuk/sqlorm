mod additions;
mod bind;
mod column;
pub mod condition;
use std::fmt::Debug;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
use crate::driver::Driver;
use crate::format_alised_col_name;
pub use additions::JoinSpec;
pub use additions::JoinType;
pub use additions::OrderBySpec;
pub use bind::BindValue;
pub use column::Column;
pub use condition::Condition;
use sqlx::QueryBuilder;

/// Quote identifiers appropriately for the target database
/// Both PostgreSQL and SQLite support double quotes for identifiers
pub fn with_quotes(s: &str) -> String {
    // Double quotes work for both PostgreSQL and SQLite
    // This ensures consistent behavior across databases
    format!("\"{}\"", s)
}

/// Query builder for composing SELECT statements with optional joins and filters.
pub struct QB<T> {
    /// Base table information and selected columns.
    pub base: TableInfo,

    /// Eager joins that project columns from related tables.
    pub eager: Vec<JoinSpec>,
    /// Batch joins for has-many relations.
    pub batch: Vec<JoinSpec>,

    /// WHERE clause conditions combined with AND.
    pub filters: Vec<Condition>,
    pub order_by: Vec<OrderBySpec>,

    pub limit: Option<i32>,
    pub offset: Option<i32>,

    _marker: std::marker::PhantomData<T>,
}
#[derive(Clone, Debug)]
/// Static information about a table used to build queries.
pub struct TableInfo {
    /// Database table name.
    pub name: &'static str,
    /// SQL alias to use for the table in the query.
    pub alias: String,
    /// Columns to project for this table.
    pub columns: Vec<&'static str>,
}

impl<T> QB<T> {
    pub fn new(base: TableInfo) -> QB<T> {
        QB {
            base,
            eager: Vec::new(),
            order_by: Vec::new(),
            batch: Vec::new(),
            filters: Vec::new(),
            _marker: std::marker::PhantomData,
            limit: None,
            offset: None,
        }
    }

    pub fn filter(mut self, cond: Condition) -> Self {
        self.filters.push(cond);
        self
    }

    fn apply_projections(&self, builder: &mut QueryBuilder<'static, Driver>) {
        let mut projections = Vec::new();

        for col in &self.base.columns {
            let field = format!("{}.{}", self.base.alias, col);
            let as_field = format_alised_col_name(&self.base.alias, col);
            projections.push(format!("{} AS {}", field, as_field));
        }

        for join in &self.eager {
            for col in &join.foreign_table.columns {
                let field = format!("{}.{}", join.foreign_table.alias, col);
                let as_field = format_alised_col_name(&join.foreign_table.alias, col);
                projections.push(format!("{} AS {}", field, as_field));
            }
        }

        builder.push(projections.join(", "));

        builder.push(" ");
    }

    fn apply_from_clause(&self, builder: &mut QueryBuilder<'static, Driver>) {
        builder.push(format!(
            "FROM {} AS {}",
            with_quotes(self.base.name),
            self.base.alias
        ));

        builder.push(" ");
    }

    fn apply_joins(&self, builder: &mut QueryBuilder<'static, Driver>) {
        let mut joins = String::new();

        for join in &self.eager {
            let other_table = format!(
                "{} AS {}",
                with_quotes(join.foreign_table.name),
                join.foreign_table.alias
            );

            let jt = match join.join_type {
                JoinType::Inner => "INNER JOIN",
                JoinType::Left => "LEFT JOIN",
            };

            let on_base = format!("{}.{}", self.base.alias, join.on.0);
            let on_other = format!("{}.{}", join.foreign_table.alias, join.on.1);

            joins.push_str(&format!(
                " {} {} ON {} = {}",
                jt, other_table, on_base, on_other
            ));
        }

        builder.push(joins);
    }

    fn apply_limit<'args>(&self, builder: &mut QueryBuilder<'args, Driver>) {
        if let Some(l) = self.limit {
            builder.push(" LIMIT ");
            builder.push_bind(l);
        }
    }

    fn apply_offset<'args>(&self, builder: &mut QueryBuilder<'args, Driver>) {
        if let Some(o) = self.offset {
            #[cfg(feature = "sqlite")]
            if let None = self.limit {
                builder.push(" LIMIT ");
                builder.push_bind(-1);
            }
            builder.push(" OFFSET ");
            builder.push_bind(o);
        }
    }

    fn apply_filters(&self, builder: &mut QueryBuilder<'static, Driver>) {
        if !self.filters.is_empty() {
            builder.push(" WHERE ");

            for (i, cond) in self.filters.iter().enumerate() {
                if i > 0 {
                    builder.push(" AND ");
                }

                let mut parts = cond.sql.split('?');
                if let Some(first) = parts.next() {
                    builder.push(first);
                }

                for (val, part) in cond.values.iter().zip(parts) {
                    val.bind(builder);
                    builder.push(part);
                }
            }
        }
    }

    fn apply_order_by(&self, builder: &mut QueryBuilder<'static, Driver>) {
        if self.order_by.is_empty() {
            return;
        }

        builder.push(" ORDER BY ");

        for (i, spec) in self.order_by.iter().enumerate() {
            if i > 0 {
                builder.push(", ");
            }
            builder.push(format!("{} {}", spec.column, spec.order));
        }
    }

    pub fn build_query(&self) -> QueryBuilder<'static, Driver> {
        let mut builder = QueryBuilder::new("SELECT ");

        self.apply_projections(&mut builder);
        self.apply_from_clause(&mut builder);
        self.apply_joins(&mut builder);
        self.apply_filters(&mut builder);
        self.apply_order_by(&mut builder);
        self.apply_limit(&mut builder);
        self.apply_offset(&mut builder);

        builder
    }

    pub fn to_sql(&self) -> String {
        self.build_query().sql().to_string()
    }
}
