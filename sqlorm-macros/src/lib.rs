use proc_macro::TokenStream;
use quote::ToTokens;
use syn::ItemStruct;
use syn::parse_macro_input;

mod naming;
mod traits;

use crate::entity::EntityStruct;
mod entity;
mod qb;
mod sql;

mod attrs;
mod gen_columns;
mod relations;

#[proc_macro_derive(Entity, attributes(sql))]
pub fn entity(input: TokenStream) -> TokenStream {
    let es = parse_macro_input!(input as EntityStruct);
    entity::handle(es).into()
}

#[proc_macro_attribute]
pub fn table(_: TokenStream, input: TokenStream) -> TokenStream {
    let model = parse_macro_input!(input as ItemStruct);
    let model = model.to_token_stream();

    quote::quote! {
        #[derive(::sqlorm::sqlx::FromRow,::sqlorm::Entity)]
        #model
    }
    .into()
}
