//! Attribute parsing for Entity macro fields.
//!
//! This module handles parsing of `#[sql(...)]` and `#[sqlx(...)]` attributes
//! on struct fields, converting them into the appropriate `EntityField` metadata
//! for code generation.

use syn::{Field, Ident, LitStr, Result, Token, parse::ParseStream};

use crate::{
    entity::{EntityField, FieldKind, TimestampKind},
    relations::{Relation, RelationType},
};

/// Parses a single struct field into an `EntityField` with all its metadata.
/// 
/// This function processes all `#[sql(...)]` and `#[sqlx(...)]` attributes on a field,
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
/// - `timestamp = "type"` - Automatic timestamp management (created_at, updated_at, deleted_at)
/// - `relation(...)` - Define relationships
/// 
/// ## `#[sqlx(...)]`
/// - `skip` - Exclude from SQL operations
pub fn parse_entity_field(field: &Field) -> Result<EntityField> {
    let mut kind = FieldKind::Regular { unique: false };
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
                    "pk" => {
                        kind = FieldKind::PrimaryKey;
                    }
                    "timestamp" => {
                        let lit: LitStr = meta.value()?.parse()?;
                        kind = FieldKind::Timestamp(match lit.value().as_str() {
                            "created_at" => TimestampKind::Created,
                            "updated_at" => TimestampKind::Updated,
                            "deleted_at" => TimestampKind::Deleted,
                            _ => return Err(meta.error("unrecognized timestamp kind")),
                        });
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
        } else if attr.path().is_ident("sqlx") {
            attr.parse_nested_meta(|meta| {
                let ident = meta
                    .path
                    .get_ident()
                    .map(|i| i.to_string())
                    .ok_or_else(|| meta.error("expected identifier"))?;

                match ident.as_str() {
                    "skip" => {
                        kind = FieldKind::Ignored;
                    }
                    _ => return Err(meta.error("unrecognized sqlx modifier")),
                }
                Ok(())
            })?;
        }
    }

    Ok(EntityField {
        ident: field.ident.clone().unwrap(),
        ty: field.ty.clone(),
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
/// ```text
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
/// ```rust
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
    if relation_ident != "relation" {
        return Err(syn::Error::new_spanned(
            relation_ident,
            "expected `relation = \"...\"`",
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
    let other_field: Ident = input.parse()?; // <-- only one ident

    Ok(Relation {
        kind: relation_type,
        other: ref_table,
        relation_name,
        on: (self_ident, other_field), // <-- tuple (self_field, other_field)
    })
}
