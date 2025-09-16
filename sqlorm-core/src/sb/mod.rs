use crate::{Condition, Connection, Driver, TableInfo};

struct SB<T> {
    /// Base table information and selected columns.
    pub base: TableInfo,
    /// Fields update
    pub fields: Option<Vec<&'static str>>,
    /// WHERE clause conditions combined with AND.
    pub filters: Vec<Condition>,
    _marker: std::marker::PhantomData<T>,
}
impl<T> SB<T> {
    pub fn new(base: TableInfo) -> SB<T> {
        SB {
            base,
            filters: Vec::new(),
            fields: None,
            _marker: std::marker::PhantomData,
        }
    }
    pub fn columns(&mut self, fields: Vec<&'static str>) -> &mut Self {
        self.fields = Some(fields);
        self
    }
    pub fn filter(mut self, cond: Condition) -> Self {
        self.filters.push(cond);
        self
    }
}
