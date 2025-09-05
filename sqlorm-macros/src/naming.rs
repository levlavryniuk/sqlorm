use syn::Ident;

pub fn executor_from_entity_ident(entity_ident: &Ident) -> Ident {
    Ident::new(&format!("{entity_ident}Executor"), entity_ident.span())
}

pub fn relations_from_entity_ident(entity_ident: &Ident) -> Ident {
    Ident::new(&format!("{entity_ident}Relations"), entity_ident.span())
}
