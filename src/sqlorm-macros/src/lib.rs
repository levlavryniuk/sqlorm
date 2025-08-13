#![warn(missing_debug_implementations)]
mod codegen;
mod schema;
mod table;
use proc_macro::TokenStream;
use quote::quote;
use schema::Table;
use syn::parse_macro_input;

#[proc_macro_derive(DeriveSchema, attributes(table, sql))]
pub fn schema(tts: TokenStream) -> TokenStream {
    let tbl = parse_macro_input!(tts as Table);
    let tbl_impl = codegen::table(&tbl);
    let queries = codegen::queries(&tbl);

    quote! {
        #tbl_impl

        #queries
    }
    .into()
}
