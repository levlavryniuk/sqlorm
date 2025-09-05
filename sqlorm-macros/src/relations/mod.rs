use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;
mod lazy;
mod validation;

use crate::EntityStruct;

pub fn relations(tbl: &EntityStruct) -> TokenStream {
    let lazy = lazy::lazy(tbl);
    quote! {
        #lazy
    }
}

#[derive(Debug, Clone)]
pub enum RelationType {
    BelongsTo,
    HasMany,
    HasOne,
}
#[derive(Debug, Clone)]
pub struct Relation {
    pub kind: RelationType,
    pub other: Ident,
    /// on my, on other
    /// e.g. `("id","owner_id")` as `user.id` `jar.owner_id`
    /// User has_many Jar
    pub on: (Ident, Ident),
    pub relation_name: String,
}
pub use validation::validate_relations;
