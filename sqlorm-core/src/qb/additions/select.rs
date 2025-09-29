use crate::selectable::Selectable;

impl<T> crate::QB<T> {
    pub fn select<'a, S: Selectable>(mut self, cols: S) -> crate::QB<S::Row> {
        let cols = cols.collect();
        if cols.is_empty() {
            panic!("Cannot select empty column list. At least one column must be specified.");
        }
        self.base.columns = cols;
        crate::QB {
            base: self.base,
            eager: self.eager,
            batch: self.batch,
            order_by: self.order_by,
            limit: self.limit,
            offset: self.offset,
            filters: self.filters,
            _marker: std::marker::PhantomData,
        }
    }
}
