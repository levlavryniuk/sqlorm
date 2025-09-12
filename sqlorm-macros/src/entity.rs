use crate::{
    qb,
    traits::{self},
};
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

/// Represents a single field in an entity struct during macro processing.
///
/// Contains all the metadata needed to generate appropriate SQL methods and
/// query builder integration for the field.
#[derive(Debug, Clone)]
pub struct EntityField {
    /// The Rust identifier name (e.g., `id`, `email`, `created_at`)
    pub ident: Ident,
    /// The Rust type of the field (e.g., `i64`, `String`, `DateTime<Utc>`)
    pub ty: Type,
    /// The semantic kind of field (primary key, timestamp, regular, etc.)
    pub kind: FieldKind,
    /// Associated relationships if any (has_many, belongs_to, etc.)
    pub relations: Option<Vec<relations::Relation>>,
}

/// Categorizes the semantic meaning of an entity field for code generation.
///
/// This determines what kind of special handling the field receives in generated methods.
#[derive(Debug, Clone)]
pub enum FieldKind {
    /// Primary key field marked with `#[sql(pk)]`
    PrimaryKey,
    /// Timestamp field with automatic management
    Timestamp(TimestampKind),
    /// Field excluded from SQL operations via `#[sql(skip)]`
    Ignored,
    /// Regular database field
    Regular {
        /// Whether the field is unique (generates `find_by_*` methods)
        unique: bool,
    },
}

/// Specifies the type of automatic timestamp management.
///
/// Used with `#[sql(timestamp = "...")]` attributes.
#[derive(Debug, Clone)]
pub enum TimestampKind {
    /// Field marked with `#[sql(timestamp = "created_at")]` - set on insert
    Created,
    /// Field marked with `#[sql(timestamp = "updated_at")]` - set on insert/update
    Updated,
    /// Field marked with `#[sql(timestamp = "deleted_at")]` - for soft deletes
    Deleted,
}

/// Complete representation of an entity struct during macro processing.
///
/// Contains all information needed to generate the full set of database methods,
/// query builder integration, and relationship handling.
#[derive(Debug)]
pub struct EntityStruct {
    /// The name of the Rust struct
    pub struct_ident: Ident,
    /// The database table name (from `#[table_name]` or struct name + "s")
    pub table_name: TableName,
    /// All fields in the struct
    pub fields: Vec<EntityField>,
    /// The primary key field (exactly one required)
    pub pk: EntityField,
    /// All relationships defined on this entity
    pub relations: Vec<relations::Relation>,
}

#[derive(Debug)]
pub struct TableName {
    /// Either struct name (`"User".to_lowercase()`), or user-defined value (`#[table(name = "users")]`). Always lowercase.
    pub raw: String,
    /// Usually `"__" + self.raw.to_lowercase()`
    pub alias: String,
}

impl Parse for EntityStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let derive_input: DeriveInput = input.parse()?;
        let struct_ident = derive_input.ident.clone();
        let mut table_name_raw = struct_ident.to_string().to_lowercase();

        for attr in &derive_input.attrs {
            if attr.path().is_ident("sql") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("name") {
                        let lit: syn::LitStr = meta.value()?.parse()?;
                        table_name_raw = lit.value();
                        return Ok(());
                    } else {
                        return Ok(());
                    }
                })?;
            }
        }
        let alias = format!("__{}", table_name_raw);
        let table_name = TableName {
            raw: table_name_raw,
            alias,
        };

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
    /// Returns true if this field is the primary key.
    pub fn is_pk(&self) -> bool {
        matches!(self.kind, FieldKind::PrimaryKey)
    }

    /// Returns true if this field is unique (either primary key or marked as unique).
    ///
    /// Unique fields generate `find_by_*` methods.
    pub fn is_unique(&self) -> bool {
        match self.kind {
            FieldKind::PrimaryKey => true,
            FieldKind::Regular { unique } => unique,
            _ => false,
        }
    }

    /// Returns true if this field should be ignored in SQL operations.
    ///
    /// Ignored fields are typically used for computed properties or relationships
    /// that are loaded separately.
    pub fn is_ignored(&self) -> bool {
        matches!(self.kind, FieldKind::Ignored)
    }
}
