use proc_macro2::TokenStream;
use quote::quote;
use sqlx::query_as;
use syn::Ident;

use crate::Table;

pub fn queries(tbl: &Table) -> TokenStream {
    let struct_name = &tbl.ident;
    let pk = tbl.primary_key.name.field();
    let s = pk.span();
    let val = pk.to_string();
    let pk_type = &tbl.primary_key.ty;
    let fn_name = format!("find_by_{}", &val);

    let find_by_pk = Ident::new(&fn_name, s);

    let q = quote! {
        impl #struct_name {
            pub fn #find_by_pk<T: sqlx::Database>(conn: &mut sqlx::Pool<T>,primary_key: #pk_type)->#struct_name{
                todo!()
            }
        }
    };
    println!("{q}");
    q
}
