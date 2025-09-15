//! SQL method generation for Entity macros.
//!
//! This module coordinates the generation of all SQL-related methods for entities,
//! including CRUD operations and query methods.

use proc_macro2::TokenStream;
use quote::quote;

use crate::entity::EntityStruct;

mod find;
mod save;

/// Generates all SQL methods for an entity.
///
/// Combines the generated code from:
/// - `save::save()` - insert, update, save methods
/// - `find::find()` - find_by_* methods for unique fields
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

