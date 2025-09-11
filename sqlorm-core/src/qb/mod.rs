mod bind;
mod column;
pub mod condition;
use std::fmt::Debug;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
use crate::driver::{Driver, Row};
pub use bind::BindValue;
pub use column::Column;
pub use condition::Condition;
use sqlx::FromRow;
use sqlx::QueryBuilder;

/// Quote identifiers appropriately for the target database
/// Both PostgreSQL and SQLite support double quotes for identifiers
pub fn with_quotes(s: &str) -> String {
    // Double quotes work for both PostgreSQL and SQLite
    // This ensures consistent behavior across databases
    format!("\"{}\"", s)
}

#[derive(Debug)]
/// Query builder for composing SELECT statements with optional joins and filters.
pub struct QB<T: std::fmt::Debug> {
    /// Base table information and selected columns.
    pub base: TableInfo,
    /// Eager joins that project columns from related tables.
    pub eager: Vec<JoinSpec>,
    /// Batch joins for has-many relations.
    pub batch: Vec<JoinSpec>,
    /// WHERE clause conditions combined with AND.
    pub filters: Vec<Condition>,
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

#[derive(Clone, Debug)]
/// Join type for related tables.
pub enum JoinType {
    Inner,
    Left,
}

#[derive(Clone, Debug)]
/// Specification for joining a related table.
pub struct JoinSpec {
    /// The join type.
    pub join_type: JoinType,
    /// Relation name.
    pub relation_name: &'static str,
    /// The joined table metadata.
    pub foreign_table: TableInfo,
    /// Join key mapping as (base_pk, foreign_fk).
    pub on: (&'static str, &'static str),
}

impl<T: std::fmt::Debug> QB<T> {
    pub fn new(base: TableInfo) -> QB<T> {
        QB {
            base,
            eager: Vec::new(),
            batch: Vec::new(),
            filters: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn join_eager(mut self, spec: JoinSpec) -> Self {
        self.eager.push(spec);
        self
    }

    pub fn join_batch(mut self, spec: JoinSpec) -> Self {
        self.batch.push(spec);
        self
    }

    pub fn select<'a, Out: Debug + FromRow<'a, Row>>(mut self, cols: Vec<&'static str>) -> QB<Out> {
        if cols.is_empty() {
            panic!("Cannot select empty column list. At least one column must be specified.");
        }
        self.base.columns = cols;
        QB {
            base: self.base,
            eager: self.eager,
            batch: self.batch,
            filters: self.filters,
            _marker: std::marker::PhantomData,
        }
    }

    fn build_projections(&self) -> Vec<String> {
        let mut projections = Vec::new();

        for col in &self.base.columns {
            projections.push(format!(
                "{}.{} AS {}{}",
                self.base.alias, col, self.base.alias, col
            ));
        }

        for join in &self.eager {
            for col in &join.foreign_table.columns {
                projections.push(format!(
                    "{}.{} AS {}{}",
                    join.foreign_table.alias, col, join.foreign_table.alias, col
                ));
            }
        }

        projections
    }

    fn build_from_clause(&self) -> String {
        format!(
            "FROM {} AS {}",
            with_quotes(self.base.name),
            self.base.alias
        )
    }

    pub fn filter(mut self, cond: Condition) -> Self {
        self.filters.push(cond);
        self
    }

    fn build_joins(&self) -> String {
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

        joins
    }

    pub fn build_query(&self) -> QueryBuilder<'static, Driver> {
        let projections = self.build_projections().join(", ");
        let from_clause = self.build_from_clause();
        let joins = self.build_joins();

        let mut builder = QueryBuilder::new("SELECT ");
        builder.push(projections);
        builder.push(" ");
        builder.push(from_clause);
        builder.push(" ");
        builder.push(joins);

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
                    val.bind(&mut builder);
                    builder.push(part);
                }
            }
        }

        builder
    }

    pub fn to_sql(&self) -> String {
        self.build_query().sql().to_string()
    }
}
