use crate::schema::Table;
use proc_macro2::TokenStream;
use quote::quote;

pub fn table(table: &Table) -> TokenStream {
    let struct_name = &table.ident;
    let name = &table.table_id.name;
    quote! (
        impl #struct_name{
            pub fn table_name()->String{
                #name.to_string()
            }
        }
    )
}
