use crate::EntityStruct;
// use rel::Relations
// trait Relations {
//  pub fn with_jars()
//  pub fn with_donations()
// }
mod executor_trait;
// User::query().with_jars()
mod relations_trait;

pub fn qb(es: &EntityStruct) -> proc_macro2::TokenStream {
    let s_ident = &es.struct_ident;
    let relations_trait = relations_trait::relations_trait(es);
    let executor = executor_trait::executor_trait(es);

    quote::quote! {
        #relations_trait

        #executor

        impl #s_ident {
            pub fn query() -> macros_core::QB<#s_ident> {
                macros_core::QB::new(<#s_ident as macros_core::Table>::table_info())
            }
        }
    }
}
