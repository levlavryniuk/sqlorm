//! SQL method generation for Entity macros.
//!
//! This module coordinates the generation of all SQL-related methods for entities,
//! including CRUD operations and query methods.

use proc_macro2::TokenStream;
use quote::quote;

use crate::entity::EntityStruct;

mod delete;
mod find;
mod save;

pub fn sql(es: &EntityStruct) -> TokenStream {
    let save = save::save(es);
    let delete = delete::delete(es);
    let _find_unique = quote! {};
    #[cfg(feature = "extra-traits")]
    let _find_unique = find::find_unique(es);

    quote! {
        #save
        #_find_unique
        #delete
    }
}
