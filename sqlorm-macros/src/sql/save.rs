//! Save operation generation for Entity macros.
//!
//! This module generates the `insert`, `update`, and `save` methods for entities,
//! handling automatic timestamp management and SQL generation for CRUD operations.

use proc_macro2::TokenStream;
use quote::quote;
use sqlorm_core::with_quotes;
use syn::{Ident, Type};

use crate::{
    entity::{EntityStruct, FieldKind, TimestampKind},
    sql::{generate_placeholders, generate_single_placeholder},
};

/// Checks if a type is a UUID type that should be auto-generated.
///
/// Recognizes common UUID type patterns:
/// - `Uuid`
/// - `sqlx::types::Uuid`
/// - `::sqlx::types::Uuid`
fn is_uuid_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            let path_str = quote!(#type_path).to_string().replace(' ', "");
            path_str == "Uuid"
                || path_str == "sqlx::types::Uuid"
                || path_str == "::sqlx::types::Uuid"
                || path_str == "uuid::Uuid"
                || path_str == "::uuid::Uuid"
        }
        _ => false,
    }
}

/// Generates `insert`, `update`, and `save` method implementations for an entity.
///
/// Creates three methods:
/// - `insert(self, executor)` - Forces an INSERT, auto-populates created_at/updated_at
/// - `update(self, executor)` - Forces an UPDATE, auto-updates updated_at
/// - `save(self, executor)` - INSERT if primary key is default, UPDATE otherwise
///
/// All methods:
/// - Take ownership of self to force `let user = user.save()` pattern
/// - Use `RETURNING *` to get the complete updated record
/// - Return a new instance with updated data
/// - Handle timestamp fields automatically based on their `#[sql(timestamp(field_name, factory_fn()))]` attributes
/// - Return `sqlx::Result<Self>` for error handling
///
/// # Generated SQL Examples
///
/// For a `User` entity:
/// ```sql
/// -- INSERT
/// INSERT INTO "users" (email, name, created_at, updated_at)
/// VALUES (?, ?, ?, ?)
/// RETURNING *
///
/// -- UPDATE  
/// UPDATE "users"
/// SET email = ?, name = ?, updated_at = ?
/// WHERE id = ?
/// RETURNING *
/// ```
pub fn save(es: &EntityStruct) -> TokenStream {
    let s_ident = &es.struct_ident;
    let table_name = &with_quotes(&es.table_name.raw);

    let pk_field = &es.pk;
    let pk_ident = &pk_field.ident;
    let pk_type = &pk_field.ty;

    let insert_fields: Vec<&Ident> = es
        .fields
        .iter()
        .filter(|f| !f.is_ignored())
        .filter(|f| !f.is_pk() || is_uuid_type(&f.ty))
        .map(|f| &f.ident)
        .collect();

    let insert_names: Vec<String> = insert_fields
        .iter()
        .map(|id| id.to_string().to_lowercase())
        .collect();

    let non_pk_fields: Vec<&Ident> = es
        .fields
        .iter()
        .filter(|f| !f.is_pk() && !f.is_ignored())
        .map(|f| &f.ident)
        .collect();

    let non_pk_names: Vec<String> = non_pk_fields
        .iter()
        .map(|id| id.to_string().to_lowercase())
        .collect();

    let insert_sql = {
        let cols = insert_names.join(", ");
        let placeholders = (1..=insert_fields.len())
            .map(|i| format!("${}", i))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
            table_name, cols, placeholders
        )
    };

    let update_sql = {
        let assigns = non_pk_names
            .iter()
            .enumerate()
            .map(|(i, col)| format!("{} = ${}", col, i + 1))
            .collect::<Vec<_>>()
            .join(", ");
        let where_clause = format!(
            "{} = ${}",
            pk_ident.to_string().to_lowercase(),
            non_pk_fields.len() + 1
        );
        format!(
            "UPDATE {} SET {} WHERE {} RETURNING *",
            table_name, assigns, where_clause
        )
    };

    let created_assign = es
        .fields
        .iter()
        .find(|f| matches!(f.kind, FieldKind::Timestamp(TimestampKind::Created { .. })))
        .map(|f| {
            let ident = &f.ident;
            if let FieldKind::Timestamp(TimestampKind::Created { factory }) = &f.kind {
                quote! { self.#ident = #factory; }
            } else {
                unreachable!()
            }
        });

    let updated_assign_insert = es
        .fields
        .iter()
        .find(|f| matches!(f.kind, FieldKind::Timestamp(TimestampKind::Updated { .. })))
        .map(|f| {
            let ident = &f.ident;
            if let FieldKind::Timestamp(TimestampKind::Updated { factory }) = &f.kind {
                quote! { self.#ident = #factory; }
            } else {
                unreachable!()
            }
        });

    let updated_assign_update = updated_assign_insert.clone();

    let uuid_assigns: Vec<TokenStream> = es
        .fields
        .iter()
        .filter(|f| !f.is_ignored() && is_uuid_type(&f.ty))
        .map(|f| {
            let ident = &f.ident;
            let ty = &f.ty;
            quote! {
                #[cfg(feature = "uuid")]
                if <#ty as Default>::default() == self.#ident {
                    self.#ident = uuid::Uuid::new_v4();
                }
            }
        })
        .collect();

    quote! {
        #[automatically_derived]
        impl #s_ident {
            pub async fn insert<'a, E>(mut self, executor: E) -> sqlx::Result<Self>
            where
                E: ::sqlorm::sqlx::Executor<'a, Database = ::sqlorm::Driver>,
            {
                #(#uuid_assigns)*
                #created_assign
                #updated_assign_insert

                #[cfg(feature = "compile-checked")]
                {
                    ::sqlorm::sqlx::query_as!(
                        #s_ident,
                        #insert_sql,
                        #(self.#insert_fields),*
                    )
                    .fetch_one(executor)
                    .await
                }
                #[cfg(not(feature = "compile-checked"))]
                {
                    let mut query = ::sqlorm::sqlx::query_as::<_, #s_ident>(#insert_sql);
                    #(query = query.bind(&self.#insert_fields);)*
                    query.fetch_one(executor).await
                }
            }

            pub async fn update<'a, E>(mut self, executor: E) -> sqlx::Result<Self>
            where
                E: ::sqlorm::sqlx::Executor<'a, Database = ::sqlorm::Driver>,
            {
                #updated_assign_update
                #[cfg(feature = "compile-checked")]
                {
                    ::sqlorm::sqlx::query_as!(
                        #s_ident,
                        #update_sql,
                        #(self.#non_pk_fields),*,
                        self.#pk_ident
                    )
                    .fetch_one(executor)
                    .await
                }
                #[cfg(not(feature = "compile-checked"))]
                {
                    let mut query = ::sqlorm::sqlx::query_as::<_, #s_ident>(#update_sql);
                    #(query = query.bind(&self.#non_pk_fields);)*
                    query = query.bind(&self.#pk_ident);
                    query.fetch_one(executor).await
                }
            }

            pub async fn save<'a, E>(self, executor: E) -> sqlx::Result<Self>
            where
                E: ::sqlorm::sqlx::Executor<'a, Database = ::sqlorm::Driver>,
            {
                if <#pk_type as Default>::default() == self.#pk_ident {
                    self.insert(executor).await
                } else {
                    self.update(executor).await
                }
            }
        }
    }
}
