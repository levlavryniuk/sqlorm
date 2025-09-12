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
                let on_field = &r.on.1.to_string();

                let fn_ident = Ident::new(relation_name, Span::call_site());
                let pk_ident = &pk.ident;
                let ref_table_ident = other;

                let placeholder = generate_single_placeholder(1);

                Some(quote! {
                    pub async fn #fn_ident<'a, E>(
                        &self,
                        executor: E
                    ) -> ::sqlorm::sqlx::Result<Vec<#ref_table_ident>>
                    where
                        E: ::sqlorm::sqlx::Executor<'a, Database = sqlorm::Driver>
                    {
                        use ::sqlorm::Table;
                        let table_name = #other::TABLE_NAME;
                        let sql = format!(
                            "SELECT * FROM {} WHERE {} = {}",
                            table_name, #on_field, #placeholder
                        );
                        let rows = ::sqlorm::sqlx::query_as::<_, #ref_table_ident>(&sql)
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
        #[automatically_derived]
        impl #entity {
            #(#has_many_rel)*
        }
    }
}
