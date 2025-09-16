use crate::EntityStruct;

// This generates methods like
// #ident::update(user).fields(vec![]).execute(&pool)
// struct StatementBuilder{
//      base:
// }
// trait UpdateStmt{
//  update() -> UpdateStmt
// }
pub fn update(es: &EntityStruct) -> proc_macro2::TokenStream {
    let ident = &es.struct_ident;
    quote::quote! {}
}
