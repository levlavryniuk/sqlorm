use crate::EntityStruct;

mod delete_executor;
mod update_executor;

pub fn executor(es: &EntityStruct) -> proc_macro2::TokenStream {
    let update_executor = update_executor::executor(es);
    let delete_executor = delete_executor::executor(es);

    quote::quote! {

        #update_executor

        #delete_executor

    }
}
