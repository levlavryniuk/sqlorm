use proc_macro2::TokenStream;
use quote::quote;
use sqlorm_core::with_quotes;

use crate::{EntityStruct, entity::EntityField};

pub fn table(es: &EntityStruct) -> TokenStream {
    let struct_ident = &es.struct_ident;
    let name = &es.table_name.raw;
    let alias = &es.table_name.alias;
    let sql_name = with_quotes(&name);
    let aliased_sql_name = with_quotes(&format!("{}{}", alias, name));
    let pk = &es.pk;
    let pk_name = &pk.name;

    let fields: Vec<&EntityField> = es.fields.iter().filter(|f| !f.is_ignored()).collect();

    let field_names: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();

    quote! {
        #[automatically_derived]
        impl ::sqlorm::Table for #struct_ident {
            const TABLE_NAME: &'static str = #name;
            const SQL_NAME: &'static str = #sql_name;
            const ALIASED_SQL_NAME: &'static str = #aliased_sql_name;

            const PK: &'static str = #pk_name;
            const COLUMNS: &'static [&'static str] = &[#(#field_names),*];

            fn table_info() -> ::sqlorm::TableInfo {
                ::sqlorm::TableInfo {
                    name: Self::TABLE_NAME,
                    alias: #alias.to_string(),
                    columns: Self::COLUMNS.to_vec(),
                }
            }

        }
    }
}
