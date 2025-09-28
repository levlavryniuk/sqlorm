use crate::EntityStruct;
use quote::quote;

mod executor;

pub fn sb(es: &EntityStruct) -> proc_macro2::TokenStream {
    let executor = executor::executor(es);
    let s_ident = &es.struct_ident;

    quote! {
        #executor

        #[automatically_derived]
        impl #s_ident {
            pub fn update(self) -> ::sqlorm::SB<#s_ident,::sqlorm::Update> {
                ::sqlorm::SB::new(<#s_ident as ::sqlorm::Table>::table_info(), self)
            }
        }

        #[automatically_derived]
        impl #s_ident {
            pub fn delete(self) -> ::sqlorm::SB<#s_ident,::sqlorm::Delete> {
                ::sqlorm::SB::new(<#s_ident as ::sqlorm::Table>::table_info(), self)
            }
        }
    }
}
