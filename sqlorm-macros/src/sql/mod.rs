use proc_macro2::TokenStream;
use quote::quote;

use crate::entity::EntityStruct;

mod find;
mod find_all;
mod save;

pub fn sql(es: &EntityStruct) -> TokenStream {
    let save = save::save(es);
    let find = find::find(es);
    let find_all = find_all::find_all(es);

    quote! {
        #save
        #find
        #find_all
    }
}
fn with_quotes(s: &str) -> String {
    format!("\"{}\"", s)
}
