use crate::{QB, TableInfo};

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

impl<T> QB<T> {
    pub fn join_eager(mut self, spec: JoinSpec) -> Self {
        self.eager.push(spec);
        self
    }

    pub fn join_batch(mut self, spec: JoinSpec) -> Self {
        self.batch.push(spec);
        self
    }
}
