use crate::EntityStruct;
use quote::quote;

mod executor;

pub fn sb(es: &EntityStruct) -> proc_macro2::TokenStream {
    let executor = executor::executor(es);

    quote! {

        #executor
    }
}
