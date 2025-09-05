mod belongs_to;
mod has_many;

use proc_macro2::TokenStream;

use crate::{
    EntityStruct,
    relations::lazy::{belongs_to::belongs_to, has_many::has_many},
};

pub fn lazy(es: &EntityStruct) -> TokenStream {
    let bt = belongs_to(es);
    let hm = has_many(es);
    quote::quote! {#bt #hm}
}
