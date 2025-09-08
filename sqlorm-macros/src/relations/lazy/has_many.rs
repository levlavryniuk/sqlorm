use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::{EntityStruct, relations::RelationType, sql::generate_single_placeholder};

pub fn has_many(tbl: &EntityStruct) -> TokenStream {
    let entity = &tbl.struct_ident;
    let pk = &tbl.pk;

    let has_many_rel: Vec<TokenStream> = tbl
        .relations
        .iter()
        .filter_map(|r| match r.kind {
            RelationType::HasMany => {
                let relation_name = &r.relation_name;
                let other = &r.other;
                let on_field = &r.on;

                let fn_ident = Ident::new(relation_name, Span::call_site());
                let pk_ident = &pk.ident;
                let ref_table_ident = other;

                let placeholder = generate_single_placeholder(1);
                let sql = format!("SELECT * FROM {} WHERE {} = {}", ref_table_ident, on_field.1, placeholder);

                Some(quote! {
                    pub async fn #fn_ident<'a, E>(
                        &self,
                        executor: E
                    ) -> Result<Vec<#ref_table_ident>, sqlx::Error>
                    where
                        E: sqlx::Executor<'a, Database = sqlorm::Driver>
                    {
                        let rows = sqlx::query_as::<_, #ref_table_ident>(#sql)
                            .bind(&self.#pk_ident)
                            .fetch_all(executor)
                            .await?;

                        Ok(rows)
                    }
                })
            }
            _ => None,
        })
        .collect();

    quote! {
        impl #entity {
            #(#has_many_rel)*
        }
    }
}
