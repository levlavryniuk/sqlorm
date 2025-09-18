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

        #[automatically_derived]
        impl #s_ident {
            pub fn query() -> ::sqlorm::QB<#s_ident> {
                ::sqlorm::QB::new(<#s_ident as ::sqlorm::Table>::table_info())
            }
        }

        #[automatically_derived]
        impl #s_ident {
            pub fn update(self) -> ::sqlorm::SB<#s_ident> {
                ::sqlorm::SB::new(<#s_ident as ::sqlorm::Table>::table_info(), self)
            }
        }
    }
}
