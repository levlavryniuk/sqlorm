use crate::EntityStruct;
pub fn executor(es: &EntityStruct) -> proc_macro2::TokenStream {
    let ident = &es.struct_ident;
    let declaration = declaration(es);
    let implementation = implementation(es);

    quote::quote! {

        #impl ::sqlorm::StatementExecutor for ::sqlorm::SB<#ident> {
            #implementation
        }

    }
}

pub fn implementation(es: &EntityStruct) -> proc_macro2::TokenStream {
    quote::quote! {
        async fn execute<'a,E: ::sqlorm::sqlx::Acquire<'a,Database = ::sqlorm::Driver>>(self,acquirer: E)->::sqlorm::sqlx::Result<()>{
            let fields = if let Some(f) = self.fields{
                f
            } else {
                self.base.columns
            }
            let sql = format!("update {},",&self.base.name)

        }
    }
}
