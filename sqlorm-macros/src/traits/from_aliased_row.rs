use crate::{EntityStruct, entity::EntityField, traits::aliased_table_name};
use quote::quote;

pub fn from_aliased_row(es: &EntityStruct) -> proc_macro2::TokenStream {
    let name = &es.struct_ident;
    let alias = aliased_table_name(&es.table_name);

    let fields: Vec<&EntityField> = es.fields.iter().filter(|f| !f.is_ignored()).collect();
    let field_idents: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let col_names: Vec<_> = fields.iter().map(|f| f.ident.to_string()).collect();

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
                        #field_idents: row.try_get::<#field_types, &str>(&format!("{}{}", #alias, #col_names))?
                    ),*,
                    #default_part
                })
            }
        }
    }
}
