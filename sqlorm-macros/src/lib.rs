use proc_macro::TokenStream;
use syn::ItemStruct;
use syn::parse_macro_input;

mod naming;
mod traits;

use crate::entity::EntityStruct;
mod entity;
mod qb;
mod sql;

mod attrs;
mod gen_columns;
mod relations;

#[proc_macro_derive(Entity, attributes(sql))]
pub fn entity(input: TokenStream) -> TokenStream {
    let es = parse_macro_input!(input as EntityStruct);
    entity::handle(es).into()
}

#[proc_macro_attribute]
pub fn table(args: TokenStream, input: TokenStream) -> TokenStream {
    let model = parse_macro_input!(input as ItemStruct);

    let table_name = if args.is_empty() {
        model.ident.to_string().to_lowercase()
    } else {
        let meta_list: syn::punctuated::Punctuated<syn::MetaNameValue, syn::Token![,]> =
            syn::parse_macro_input!(args with syn::punctuated::Punctuated::parse_terminated);

        let mut table_name = model.ident.to_string().to_lowercase();
        for meta in meta_list {
            if meta.path.is_ident("name") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = meta.value
                {
                    table_name = lit_str.value();
                    break;
                }
            }
        }
        table_name
    };

    quote::quote! {
        #[derive(::sqlorm::sqlx::FromRow,::sqlorm::Entity)]
        #[sql(name = #table_name)]
        #model
    }
    .into()
}
