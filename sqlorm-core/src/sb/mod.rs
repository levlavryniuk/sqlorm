use crate::{Condition, TableInfo, selectable::Selectable};

pub struct Update;
pub struct Delete;
pub struct Insert;

pub struct SB<T, Stage> {
    /// Base table information and selected columns.
    pub base: TableInfo,
    /// Fields update
    pub fields: Option<Vec<&'static str>>,
    /// WHERE clause conditions combined with AND.
    pub filters: Vec<Condition>,
    /// The entity to operate on
    pub entity: T,
    _marker: std::marker::PhantomData<Stage>,
}

impl<T, Stage> SB<T, Stage> {
    pub fn new(base: TableInfo, entity: T) -> SB<T, Stage> {
        SB {
            base,
            filters: Vec::new(),
            fields: None,
            entity,
            _marker: std::marker::PhantomData,
        }
    }
}
impl<T> SB<T, Update> {
    pub fn columns(mut self, fields: impl Selectable) -> Self {
        self.fields = Some(fields.collect());
        self
    }

    pub fn filter(mut self, cond: Condition) -> Self {
        self.filters.push(cond);
        self
    }
}
