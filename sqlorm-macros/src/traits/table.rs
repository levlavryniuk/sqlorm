use proc_macro2::TokenStream;
use quote::quote;

use crate::{EntityStruct, entity::EntityField, traits::aliased_table_name};

pub fn table(es: &EntityStruct) -> TokenStream {
    let struct_ident = &es.struct_ident;
    let table_name = &es.table_name;
    let pk = &es.pk;
    let pk_name = pk.ident.to_string();

    let alias = aliased_table_name(table_name);

    let fields: Vec<&EntityField> = es.fields.iter().filter(|f| !f.is_ignored()).collect();

    let field_names: Vec<String> = fields.iter().map(|f| f.ident.to_string()).collect();

    quote! {
        impl sqlorm::Table for #struct_ident {
            const TABLE_NAME: &'static str = #table_name;
            const PK: &'static str = #pk_name;
            const COLUMNS: &'static [&'static str] = &[#(#field_names),*];

            fn table_info() -> sqlorm_core::TableInfo {
                sqlorm_core::TableInfo {
                    name: Self::TABLE_NAME,
                    alias: #alias.to_string(),
                    // convert &'static [&'static str] into owned Vec<String>
                    columns: Self::COLUMNS.to_vec(),
                }
            }
        }
    }
}
