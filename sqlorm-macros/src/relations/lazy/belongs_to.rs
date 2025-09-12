use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::{
    EntityStruct,
    relations::{Relation, RelationType},
};

pub fn belongs_to(tbl: &EntityStruct) -> TokenStream {
    let entity = &tbl.struct_ident;

    let belongs_to_rel: Vec<TokenStream> = tbl
        .fields
        .iter()
        .filter_map(|f| f.relations.as_ref())
        .flat_map(|rels| rels.iter())
        .filter_map(|rel| {
            if let Relation {
                kind: RelationType::BelongsTo,
                relation_name,
                other,
                on: (self_field, _other_field),
            } = rel
            {
                let fn_ident = Ident::new(relation_name, Span::call_site());
                let const_field = Ident::new(&_other_field.to_string().to_uppercase(), other.span());
                Some(quote! {
                    pub async fn #fn_ident<'a, E>(
                        &self,
                        executor: E
                    ) -> ::sqlorm::sqlx::Result<Option<#other>>
                    where
                        E: Send + ::sqlorm::sqlx::Acquire<'a, Database = ::sqlorm::Driver>
                    {
                        #other::query().filter(#other::#const_field.eq(self.#self_field)).fetch_optional(executor).await
                    }
                })
            } else {
                None
            }
        })
        .collect();

    quote! {
        #[automatically_derived]
        impl #entity {
        #(#belongs_to_rel)*
        }
    }
}
