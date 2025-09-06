use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{entity::EntityStruct, sql::with_quotes};

pub fn find(es: &EntityStruct) -> TokenStream {
    let s_ident = &es.struct_ident;
    let table_name = with_quotes(&es.table_name);

    let unique_fields: Vec<_> = es
        .fields
        .iter()
        .filter(|f| f.is_unique() && !f.is_ignored())
        .collect();

    let methods: Vec<TokenStream> = unique_fields
        .iter()
        .map(|f| {
            let fname = &f.ident;
            let ftype = &f.ty;
            let method_name = Ident::new(&format!("find_by_{}", fname), fname.span());
            let col_name = fname.to_string();

            quote! {
                pub async fn #method_name<'a, E>(
                    executor: E,
                    value: #ftype
                ) -> sqlx::Result<Option<#s_ident>>
                where
                    E: sqlx::Executor<'a, Database = sqlorm_core::Driver>
                {
                    let query = format!(
                        "select * from {table} WHERE {col} = ?",
                        table = #table_name,
                        col = #col_name,
                    );
                    sqlx::query_as::<_, #s_ident>(&query)
                        .bind(value)
                        .fetch_optional(executor)
                        .await
                }
            }
        })
        .collect();

    quote! {
        impl #s_ident {
            #(#methods)*
        }
    }
}
