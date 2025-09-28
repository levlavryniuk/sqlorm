use proc_macro2::TokenStream;
use quote::quote;

use crate::entity::EntityStruct;

mod find;
mod save;

pub fn sql(es: &EntityStruct) -> TokenStream {
    let save = save::save(es);
    let _find_unique = quote! {};
    #[cfg(feature = "extra-traits")]
    let _find_unique = find::find_unique(es);

    quote! {
        #save
        #_find_unique
    }
}
