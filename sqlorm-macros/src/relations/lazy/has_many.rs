use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::{EntityStruct, relations::RelationType};

pub fn has_many(tbl: &EntityStruct) -> TokenStream {
    let entity = &tbl.struct_ident;

    let has_many_rel: Vec<TokenStream> = tbl
        .relations
        .iter()
        .filter_map(|r| match r.kind {
            RelationType::HasMany => {
                let relation_name = &r.relation_name;
                let other = &r.other;
                let on_field = &r.on.0;
                let const_on_field = Ident::new(&r.on.1.to_string().to_uppercase(),Span::call_site());

                let fn_ident = Ident::new(relation_name, Span::call_site());


                Some(quote! {
                    pub async fn #fn_ident<'a, E>(
                        &self,
                        executor: E
                    ) -> ::sqlorm::sqlx::Result<Vec<#other>>
                    where
                        E: ::sqlorm::sqlx::Acquire<'a, Database = sqlorm::Driver> + Send
                    {
                        #other::query().filter(#other::#const_on_field.eq(self.#on_field)).fetch_all(executor).await
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
