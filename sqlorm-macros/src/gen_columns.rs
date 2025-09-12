use proc_macro2::TokenStream;
use quote::quote;
use sqlorm_core::format_alised_col_name;
use syn::Ident;

use crate::entity::{EntityField, EntityStruct};

pub fn handle(entity: &EntityStruct) -> TokenStream {
    let struct_ident = entity.struct_ident.clone();
    let fields: Vec<&EntityField> = entity.fields.iter().filter(|f| !f.is_ignored()).collect();
    let field_count = fields.len();

    let table_alias = &entity.table_name.alias;

    let field_names: Vec<String> = fields.iter().map(|f| f.ident.to_string()).collect();

    let aliased_field_names: Vec<String> = fields
        .iter()
        .map(|f| format_alised_col_name(table_alias, &f.ident.to_string()))
        .collect();

    let field_ty: Vec<&syn::Type> = fields.iter().map(|f| &f.ty).collect();

    let const_idents: Vec<Ident> = fields
        .iter()
        .map(|f| {
            let name = f.ident.to_string().to_uppercase();
            Ident::new(&name, f.ident.span())
        })
        .collect();

    quote! {
        #[automatically_derived]
        impl #struct_ident {
            /// All column names of this entity in declaration order.
            pub const COLUMNS: [&'static str; #field_count] = [#(#field_names),*];

            #(
                /// Column reference for the `#field_names` field.
                pub const #const_idents: sqlorm::Column<#field_ty> =
                    sqlorm::Column { name: #field_names, aliased_name: #aliased_field_names, table_alias: #table_alias, _marker: std::marker::PhantomData };
            )*
        }
    }
}
