use crate::{EntityStruct, entity::EntityField};
use quote::quote;
use sqlorm_core::format_alised_col_name;

pub fn from_aliased_row(es: &EntityStruct) -> proc_macro2::TokenStream {
    let name = &es.struct_ident;
    let alias = &es.table_name.alias;

    let fields: Vec<&EntityField> = es.fields.iter().filter(|f| !f.is_ignored()).collect();
    let field_idents: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let col_names: Vec<_> = fields
        .iter()
        .map(|f| format_alised_col_name(alias, &f.name))
        .collect();

    let has_ignored = es.fields.iter().any(|f| f.is_ignored());

    let default_part = if has_ignored {
        quote! { ..Default::default() }
    } else {
        quote! {}
    };

    quote! {
        #[automatically_derived]
        impl ::sqlorm::FromAliasedRow for #name {
            fn from_aliased_row(
                row: &::sqlorm::Row,
            ) -> ::sqlorm::sqlx::Result<Self> where Self: Sized+Default {
                use ::sqlorm::sqlx::Row;
                Ok(Self {
                    #(
                        #field_idents: row.try_get::<#field_types, &str>(#col_names)?
                    ),*,
                    #default_part
                })
            }
        }
    }
}

pub fn from_row_impl(es: &EntityStruct) -> proc_macro2::TokenStream {
    let ident = &es.struct_ident;

    let fields: Vec<&EntityField> = es.fields.iter().filter(|f| !f.is_ignored()).collect();
    let field_idents: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let col_names: Vec<_> = fields.iter().map(|f| f.name.clone()).collect();

    let has_ignored = es.fields.iter().any(|f| f.is_ignored());

    let default_part = if has_ignored {
        quote! { ..Default::default() }
    } else {
        quote! {}
    };

    quote! {
        #[automatically_derived]
        impl<'r> ::sqlorm::sqlx::FromRow<'r, ::sqlorm::Row> for #ident {
            fn from_row(
                row: &'r ::sqlorm::Row
            ) -> ::std::result::Result<Self, ::sqlorm::sqlx::Error> {
                use ::sqlorm::sqlx::Row;
                Ok(Self {
                    #(
                        #field_idents: row.try_get::<#field_types, &str>(#col_names)?
                    ),*,
                    #default_part
                })
            }
        }
    }
}
