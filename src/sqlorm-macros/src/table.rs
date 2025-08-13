use syn::{
    Ident, ItemStruct, LitStr, Token,
    parse::{Parse, ParseStream},
};

#[derive(Debug, Clone)]
pub struct TableId {
    pub ident: Ident,
    pub name: String,
}
impl TryFrom<&ItemStruct> for TableId {
    type Error = syn::Error;

    fn try_from(value: &ItemStruct) -> Result<Self, Self::Error> {
        value
            .attrs
            .iter()
            .find(|a| a.path().is_ident("table"))
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    &value.ident,
                    "Schema must have #[table(name = \"...\")] attribute",
                )
            })?
            .parse_args::<TableId>()
    }
}

impl Parse for TableId {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident != "name" {
            return Err(syn::Error::new_spanned(ident, "expected `name = \"...\"`"));
        }
        input.parse::<Token![=]>()?;
        let lit: LitStr = input.parse()?;
        Ok(Self {
            name: lit.value(),
            ident,
        })
    }
}
