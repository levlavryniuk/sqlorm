use std::collections::HashSet;

use sqlorm_core::{
    column::{Column, DataColumn, TimestampColumn},
    keys::{ForeignKey, PrimaryKey},
};
use syn::{
    Error, Fields, ItemStruct, TypeNever,
    parse::{Parse, ParseStream},
};

use crate::table::TableId;

#[derive(Debug, Clone)]
pub struct Table {
    pub ident: syn::Ident,
    pub table_id: TableId,

    pub primary_key: PrimaryKey,
    pub foreign_keys: HashSet<ForeignKey>,
    pub data_columns: HashSet<DataColumn>,
    pub timestamp_columns: HashSet<TimestampColumn>,
}

impl Parse for Table {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item: ItemStruct = input.parse()?;
        let _struct_name = item.ident.to_string();
        let table_id = TableId::try_from(&item)?;

        let ident = item.ident;
        let fields = match item.fields {
            Fields::Named(n) => n,
            Fields::Unnamed(_) | Fields::Unit => {
                return Err(Error::new(
                    ident.span(),
                    format!("{ident} must use named fields in order to derive `Schema`"),
                ));
            }
        };

        let columns = fields
            .named
            .into_iter()
            .map(Column::try_from)
            .collect::<syn::Result<HashSet<Column>>>()?;

        let primary_key = {
            let primary_keys: HashSet<PrimaryKey> = columns
                .iter()
                .filter_map(|c| c.as_primary_key())
                .cloned()
                .collect();

            if primary_keys.len() > 1 {
                return Err(Error::new(
                    input.span(),
                    format!(
                        "{ident} declares more than one column as its primary key – only one is allowed"
                    ),
                ));
            }

            primary_keys.into_iter().next().ok_or(Error::new(
                input.span(),
                format!("{ident} must declare one field as its primary key (using `#[pk]`"),
            ))?
        };

        let foreign_keys = columns
            .iter()
            .filter_map(|c| c.as_foreign_key())
            .cloned()
            .collect();

        let data_columns = columns
            .iter()
            .filter_map(|c| c.as_data_column())
            .cloned()
            .collect();

        let timestamp_columns = columns
            .iter()
            .filter_map(|c| c.as_timestamp_column())
            .cloned()
            .collect();

        Ok(Self {
            table_id,
            ident,
            foreign_keys,
            primary_key,
            timestamp_columns,
            data_columns,
        })
    }
}
