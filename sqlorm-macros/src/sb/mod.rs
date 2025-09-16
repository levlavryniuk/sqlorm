use crate::EntityStruct;
mod executor;

pub fn sb(es: &EntityStruct) -> proc_macro2::TokenStream {
    let ident = &es.struct_ident;
}
