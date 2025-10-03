//! Attribute parsing for Entity macro fields.
//!
//! This module handles parsing of `#[sql(...)]` attributes
//! on struct fields, converting them into the appropriate `EntityField` metadata
//! for code generation.

use syn::{Expr, Field, Ident, LitStr, Result, Token, parse::ParseStream};

use crate::{
    entity::{EntityField, FieldKind, TimestampKind},
    relations::{Relation, RelationType},
};

/// Parses a single struct field into an `EntityField` with all its metadata.
///
/// This function processes all `#[sql(...)]` attributes on a field,
/// extracting information about:
/// - Field type (primary key, unique, timestamp, etc.)
/// - Relationships (belongs_to, has_many, has_one)
/// - Whether the field should be ignored in SQL operations
///
/// # Supported Attributes
///
/// ## `#[sql(...)]`
/// - `pk` - Mark as primary key
/// - `unique` - Mark as unique (generates find_by_* methods)
/// - `timestamp(field_name, factory_fn())` - Automatic timestamp management with custom factory
/// - `relation(...)` - Define relationships
///
pub fn parse_entity_field(field: &Field) -> Result<EntityField> {
    let mut kind = FieldKind::Regular { unique: false };
    let ident = field.ident.clone().unwrap();
    let mut name = ident.to_string();
    let mut relations: Vec<Relation> = Vec::new();

    for attr in &field.attrs {
        if attr.path().is_ident("sql") {
            attr.parse_nested_meta(|meta| {
                let ident = meta
                    .path
                    .get_ident()
                    .map(|i| i.to_string())
                    .ok_or_else(|| meta.error("expected identifier"))?;

                match ident.as_str() {
                    "unique" => {
                        kind = FieldKind::Regular { unique: true };
                    }
                    "skip" => {
                        kind = FieldKind::Ignored;
                    }
                    "pk" => {
                        kind = FieldKind::PrimaryKey;
                    }
                    "rename" => {
                        let content;
                        syn::parenthesized!(content in meta.input);
                        let col: LitStr = content.parse()?;
                        name = col.value();
                    }
                    "timestamp" => {
                        let content;
                        syn::parenthesized!(content in meta.input);
                        let timestamp = parse_timestamp(&content)?;
                        kind = FieldKind::Timestamp(timestamp);
                    }
                    "relation" => {
                        let content;
                        syn::parenthesized!(content in meta.input);
                        let relation = parse_relation(&content, field.ident.clone().unwrap())?;
                        relations.push(relation);
                    }
                    _ => return Err(meta.error("unrecognized sql modifier")),
                }
                Ok(())
            })?;
        }
    }

    Ok(EntityField {
        ident,
        ty: field.ty.clone(),
        name,
        kind,
        relations: if relations.is_empty() {
            None
        } else {
            Some(relations)
        },
        // col: field.ident.clone().unwrap().to_string(),
    })
}

/// Parses a relationship attribute into a `Relation` struct.
///
/// Expected syntax:
/// ```ignore
/// #[sql(relation(TYPE -> TargetEntity, relation = "field_name", on = foreign_key))]
/// ```
///
/// Where:
/// - `TYPE` is one of: `belongs_to`, `has_many`, `has_one`
/// - `TargetEntity` is the related entity struct name
/// - `"field_name"` is the name of the field that will hold the relationship
/// - `foreign_key` is the field name that contains the foreign key
///
/// # Example
///
/// ```ignore
/// #[sql(relation(has_many -> Post, relation = "posts", on = user_id))]
/// pub id: i64,
/// ```
///
/// This creates a `has_many` relationship to `Post` entities, accessible via
/// a `posts` field, where the `Post` table has a `user_id` foreign key.
pub fn parse_relation(input: ParseStream, self_ident: Ident) -> Result<Relation> {
    let rel_type_ident: Ident = input.parse()?;
    let relation_type = match rel_type_ident.to_string().as_str() {
        "belongs_to" => RelationType::BelongsTo,
        "has_many" => RelationType::HasMany,
        "has_one" => RelationType::HasOne,
        other => {
            return Err(syn::Error::new_spanned(
                rel_type_ident,
                format!(
                    "invalid relation type `{}`. Expected one of: belongs_to, has_many, has_one",
                    other
                ),
            ));
        }
    };

    input.parse::<Token![->]>()?;
    let ref_table: Ident = input.parse()?;

    input.parse::<Token![,]>()?;
    let relation_ident: Ident = input.parse()?;
    if relation_ident != "name" {
        return Err(syn::Error::new_spanned(
            relation_ident,
            "expected `name = \"...\"`",
        ));
    }
    input.parse::<Token![=]>()?;
    let relation_val: LitStr = input.parse()?;
    let relation_name = relation_val.value();

    input.parse::<Token![,]>()?;
    let on_ident_kw: Ident = input.parse()?;
    if on_ident_kw != "on" {
        return Err(syn::Error::new_spanned(on_ident_kw, "expected `on = ...`"));
    }
    input.parse::<Token![=]>()?;
    let other_field: Ident = input.parse()?;

    Ok(Relation {
        kind: relation_type,
        other: ref_table,
        relation_name,
        on: (self_ident, other_field),
    })
}

/// Parses a timestamp attribute into a `TimestampKind`.
///
/// Expected syntax:
/// ```ignore
/// #[sql(timestamp(field_name, factory_fn()))]
/// ```
///
/// Where:
/// - `field_name` is the timestamp type: `created_at`, `updated_at`, or `deleted_at`
/// - `factory_fn()` is the expression that will be called to generate the timestamp value
///
/// # Example
///
/// ```ignore
/// #[sql(timestamp(created_at, chrono::Utc::now()))]
/// pub created_at: DateTime<Utc>,
/// ```
pub fn parse_timestamp(input: ParseStream) -> Result<TimestampKind> {
    let field_name: Ident = input.parse()?;
    input.parse::<Token![,]>()?;
    let factory: Expr = input.parse()?;

    match field_name.to_string().as_str() {
        "created_at" => Ok(TimestampKind::Created { factory }),
        "updated_at" => Ok(TimestampKind::Updated { factory }),
        "deleted_at" => Ok(TimestampKind::Deleted { factory }),
        _ => Err(syn::Error::new_spanned(
            field_name,
            "timestamp field name must be one of: created_at, updated_at, deleted_at",
        )),
    }
}
