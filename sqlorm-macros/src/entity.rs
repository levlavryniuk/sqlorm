use crate::{qb, traits};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Field, Fields, Ident, Result, Type,
    parse::{Parse, ParseStream},
};

use crate::{
    attrs::{self},
    gen_columns,
    relations::{self, validate_relations},
    sql,
};

#[derive(Debug, Clone)]
pub struct EntityField {
    /// id as in user.id
    pub ident: Ident,
    /// Uuid as in user.id
    pub ty: Type,
    /// PrimaryKey as in user.id
    pub kind: FieldKind,
    /// HasMany and stuff as in user.id
    pub relations: Option<Vec<relations::Relation>>,
    // id as in sql user.id
    // pub col: String,
}
#[derive(Debug, Clone)]
pub enum FieldKind {
    PrimaryKey,
    Timestamp(TimestampKind),
    Ignored,
    Regular { unique: bool },
}

#[derive(Debug, Clone)]
pub enum TimestampKind {
    Created,
    Updated,
    Deleted,
}

#[derive(Debug)]
pub struct EntityStruct {
    pub struct_ident: Ident,
    pub table_name: String,
    pub fields: Vec<EntityField>,
    pub pk: EntityField,
    pub relations: Vec<relations::Relation>,
}

impl Parse for EntityStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let derive_input: DeriveInput = input.parse()?;
        let struct_ident = derive_input.ident.clone();
        let mut table_name = struct_ident.to_string().to_lowercase() + "s";

        for attr in &derive_input.attrs {
            if attr.path().is_ident("table_name") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("name") {
                        let lit: syn::LitStr = meta.value()?.parse()?;
                        table_name = lit.value();
                        return Ok(());
                    }
                    Err(meta.error("unrecognized table_name attribute"))
                })?;
            }
        }

        let fields: Vec<EntityField> = match derive_input.data {
            Data::Struct(data) => match data.fields {
                Fields::Named(named) => named
                    .named
                    .into_iter()
                    .map(|f: Field| attrs::parse_entity_field(&f))
                    .collect::<Result<Vec<_>>>()?,
                _ => {
                    return Err(syn::Error::new_spanned(
                        struct_ident,
                        "Entity must have named fields",
                    ));
                }
            },
            _ => {
                return Err(syn::Error::new_spanned(
                    struct_ident,
                    "Entity can only be derived for structs",
                ));
            }
        };

        let pk = fields
            .iter()
            .find(|f| f.is_pk())
            .expect("Entity must have 1 primary key")
            .clone();

        let relations = fields
            .iter()
            .filter_map(|f| f.relations.clone())
            .reduce(|mut acc, mut r| {
                acc.append(&mut r);
                acc
            })
            .unwrap_or_default();
        validate_relations(&relations, &fields)?;

        let pk_count = fields
            .iter()
            .filter(|f| matches!(f.kind, FieldKind::PrimaryKey))
            .count();

        if pk_count == 0 {
            return Err(syn::Error::new_spanned(
                struct_ident,
                "Entity must have a primary key",
            ));
        }
        if pk_count > 1 {
            return Err(syn::Error::new_spanned(
                struct_ident,
                "Entity must have only 1 primary key",
            ));
        }

        Ok(Self {
            struct_ident,
            table_name,
            fields,
            relations,
            pk,
        })
    }
}

pub fn handle(es: EntityStruct) -> TokenStream {
    let cols = gen_columns::handle(&es);
    let sql = sql::sql(&es);
    let relations = relations::relations(&es);
    let traits = traits::traits(&es);
    let qb = qb::qb(&es);
    quote! (
        #cols

        #sql

        #relations

        #traits

        #qb
    )
}

impl EntityField {
    pub fn is_pk(&self) -> bool {
        matches!(self.kind, FieldKind::PrimaryKey)
    }

    pub fn is_unique(&self) -> bool {
        match self.kind {
            FieldKind::PrimaryKey => true,
            FieldKind::Regular { unique } => unique,
            _ => false,
        }
    }

    pub fn is_ignored(&self) -> bool {
        matches!(self.kind, FieldKind::Ignored)
    }
}
