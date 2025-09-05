use crate::EntityStruct;

mod from_aliased_row;
mod table;

pub fn traits(es: &EntityStruct) -> proc_macro2::TokenStream {
    let table = table::table(es);
    let from_aliased_row = from_aliased_row::from_aliased_row(es);
    quote::quote! {

        #table

        #from_aliased_row

    }
}
pub fn aliased_table_name(name: &str) -> String {
    format!("{}{}", name.to_lowercase(), "__")
}
