use crate::{Condition, TableInfo, selectable::Selectable};

pub struct SB<T> {
    /// Base table information and selected columns.
    pub base: TableInfo,
    /// Fields update
    pub fields: Option<Vec<&'static str>>,
    /// WHERE clause conditions combined with AND.
    pub filters: Vec<Condition>,
    /// The entity to operate on
    pub entity: T,
}
impl<T> SB<T> {
    pub fn new(base: TableInfo, entity: T) -> SB<T> {
        SB {
            base,
            filters: Vec::new(),
            fields: None,
            entity,
        }
    }

    pub fn columns(mut self, fields: impl Selectable) -> Self {
        self.fields = Some(fields.collect());
        self
    }

    pub fn filter(mut self, cond: Condition) -> Self {
        self.filters.push(cond);
        self
    }
}
