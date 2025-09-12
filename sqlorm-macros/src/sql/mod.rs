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
    let find_unique = find::find_unique(es);

    quote! {
        #save
        #[cfg(feature = "extra-traits")]
        #find_unique
    }
}

/// Generates appropriate parameter placeholders based on the enabled database feature.
/// PostgreSQL uses $1, $2, ... while SQLite uses ?
pub(crate) fn generate_placeholders(count: usize) -> Vec<String> {
    #[cfg(feature = "postgres")]
    {
        (1..=count).map(|i| format!("${}", i)).collect()
    }
    #[cfg(not(feature = "postgres"))]
    {
        (0..count).map(|_| "?".to_string()).collect()
    }
}

/// Generates a single parameter placeholder based on the enabled database feature.
/// PostgreSQL uses $n while SQLite uses ?
pub(crate) fn generate_single_placeholder(position: usize) -> String {
    #[cfg(feature = "postgres")]
    {
        format!("${}", position)
    }
    #[cfg(not(feature = "postgres"))]
    {
        let _ = position; // Suppress unused parameter warning
        "?".to_string()
    }
}
