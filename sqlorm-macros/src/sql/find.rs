use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::entity::EntityStruct;

pub fn find(es: &EntityStruct) -> TokenStream {
    let s_ident = &es.struct_ident;

    let unique_fields: Vec<_> = es
        .fields
        .iter()
        .filter(|f| f.is_unique() && !f.is_ignored())
        .collect();

    let methods: Vec<TokenStream> = unique_fields
        .iter()
        .map(|f| {
            let fname = &f.ident;
            let ftype = &f.ty;
            let method_name = Ident::new(&format!("find_by_{}", fname), fname.span());
            let col_const = Ident::new(&fname.to_string().to_uppercase(), fname.span());
            let doc_string = format!(
                "Finds a record by its {} field.\n\n\
                This method queries the database for a single record where the {} field\n\
                matches the provided value. The field must be marked as unique (either\n\
                with `#[sql(pk)]` or `#[sql(unique)]`).\n\n\
                # Parameters\n\n\
                * `executor` - A database executor (connection, pool, or transaction)\n\
                * `value` - The {} value to search for\n\n\
                # Returns\n\n\
                Returns `Ok(Some(Self))` if a matching record is found,\n\
                `Ok(None)` if no record matches, or an `sqlx::Error` if the\n\
                query fails.\n\n\
                # Example\n\n\
                ```ignore
                // Find user by {}\n\
                if let Some(user) = User::{}(&pool, value).await? {{\n\
                    println!(\"Found user: {{}}\", user.{});\n\
                }} else {{\n\
                    println!(\"No user found with {} {{}}\", value);\n\
                }}\n\
                ```",
                fname, fname, fname, fname, method_name, fname, fname
            );

            quote! {
                #[doc = #doc_string]
                pub async fn #method_name<'a, A>(
                    acquirer: A,
                    value: #ftype
                ) -> sqlx::Result<Option<#s_ident>>
                where
                    A: sqlx::Acquire<'a, Database = sqlorm::Driver>
                {
                    #s_ident::query()
                        .filter(#s_ident::#col_const.eq(value))
                        .fetch_optional(acquirer)
                        .await
                }
            }
        })
        .collect();

    quote! {
        impl #s_ident {
            #(#methods)*
        }
    }
}
