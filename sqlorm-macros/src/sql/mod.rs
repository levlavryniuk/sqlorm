//! SQL method generation for Entity macros.
//!
//! This module coordinates the generation of all SQL-related methods for entities,
//! including CRUD operations and query methods.

use proc_macro2::TokenStream;
use quote::quote;

use crate::entity::EntityStruct;

mod find;
mod find_all;
mod save;

/// Generates all SQL methods for an entity.
/// 
/// Combines the generated code from:
/// - `save::save()` - insert, update, save methods
/// - `find::find()` - find_by_* methods for unique fields 
/// - `find_all::find_all()` - find_all method
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

/// Wraps a string with double quotes for SQL identifier quoting.
/// 
/// This function is used to properly quote table and column names in generated SQL,
/// ensuring compatibility with both PostgreSQL and SQLite databases.
/// Both databases support double-quoted identifiers, making this a safe choice
/// for cross-database compatibility.
fn with_quotes(s: &str) -> String {
    format!("\"{}\"", s)
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
