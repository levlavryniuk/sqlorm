use syn::Ident;

use crate::{EntityStruct, naming::relations_from_entity_ident, qb::executor_trait::FetchVariant};

// This module is supposed to generate Relation trait for each entity
// Relations trait is supposed to contain all possible relations fo
// trait UserRelations{
//     fn with_jars(mut self)->Self
//     fn with_donations(mut self)->Self
// }
// ... later in codegen
// impl UserRelations for QB<User> {
//     fn with_jars(mut self) -> Self{
//         let spec = JoinSpec { ... };
//         self.join(spec);
//         self
//     }
//
//     fn with_donations(mut self) -> Self{
//         let spec = JoinSpec { ... };
//         self.join(spec);
//         self
//     }
// }

pub fn relations_trait(es: &EntityStruct) -> proc_macro2::TokenStream {
    let s_ident = &es.struct_ident;
    let rel_ident = relations_from_entity_ident(&es.struct_ident);
    let fn_idents = declarations(es);
    let implementations = implementations(es, &rel_ident);

    quote::quote! {
        pub trait #rel_ident {
            #(
                fn #fn_idents(self) -> ::sqlorm::QB<#s_ident>;
            )*
        }

        #implementations
    }
}

fn implementations(es: &EntityStruct, trait_name: &Ident) -> proc_macro2::TokenStream {
    let s_ident = &es.struct_ident;
    let fns: Vec<proc_macro2::TokenStream> = es
        .relations
        .iter()
        .map(|rel| {
            let fn_ident = Ident::new(&format!("with_{}", rel.relation_name), rel.other.span());
            let other = &rel.other;
            let relation_name = &rel.relation_name;
            let (on1, on2) = (&rel.on.0.to_string(), &rel.on.1.to_string());
            let fetch_variant: FetchVariant = (&rel.kind).into();

            match fetch_variant {
                FetchVariant::Eager => {
                    quote::quote! {
                        fn #fn_ident(self) -> ::sqlorm::QB<#s_ident> {
                            let join_type = ::sqlorm::JoinType::Left;
                            let foreign_table = <#other as ::sqlorm::Table>::table_info();
                            let spec = ::sqlorm::JoinSpec {
                                relation_name: #relation_name,
                                join_type,
                                foreign_table,
                                on: (#on1, #on2),
                            };
                            self.join_eager(spec)
                        }
                    }
                }
                FetchVariant::Batch => {
                    quote::quote! {
                        fn #fn_ident(self) -> ::sqlorm::QB<#s_ident> {
                            let join_type = ::sqlorm::JoinType::Left;
                            let foreign_table = <#other as ::sqlorm::Table>::table_info();
                            let spec = ::sqlorm::JoinSpec {
                                relation_name: #relation_name,
                                join_type,
                                foreign_table,
                                on: (#on1, #on2),
                            };
                            self.join_batch(spec)
                        }
                    }
                }
            }
        })
        .collect();

    quote::quote! {
        #[automatically_derived]
        impl #trait_name for ::sqlorm::QB<#s_ident> {
            #(#fns)*
        }
    }
}

fn declarations(es: &EntityStruct) -> Vec<Ident> {
    es.relations
        .iter()
        .map(|rel| format!("with_{}", &rel.relation_name))
        .map(|name| Ident::new(&name, es.struct_ident.span()))
        .collect()
}
