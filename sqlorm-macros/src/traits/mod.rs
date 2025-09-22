use crate::EntityStruct;

mod from_aliased_row;
mod table;

pub fn traits(es: &EntityStruct) -> proc_macro2::TokenStream {
    let table = table::table(es);
    let from_aliased_row = from_aliased_row::from_aliased_row(es);
    let from_row = from_aliased_row::from_row_impl(es);
    quote::quote! {

        #table

        #from_aliased_row

        #from_row

    }
}
