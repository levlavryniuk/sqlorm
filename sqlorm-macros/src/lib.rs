use proc_macro::TokenStream;
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

#[proc_macro_derive(Entity, attributes(sql, table_name))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let es = parse_macro_input!(input as EntityStruct);
    entity::handle(es).into()
}

#[proc_macro_derive(GenColumns)]
pub fn gen_columns_handler(input: TokenStream) -> TokenStream {
    let es = parse_macro_input!(input as EntityStruct);
    gen_columns::handle(&es).into()
}
