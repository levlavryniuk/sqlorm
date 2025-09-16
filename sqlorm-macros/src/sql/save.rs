//! Save operation generation for Entity macros.
//!
//! This module generates the `insert`, `update`, and `save` methods for entities,
//! handling automatic timestamp management and SQL generation for CRUD operations.

use proc_macro2::TokenStream;
use quote::quote;
use sqlorm_core::with_quotes;
use syn::{Ident, Type};

use crate::entity::{EntityStruct, FieldKind, TimestampKind};

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

    let non_pk_fields: Vec<&Ident> = es
        .fields
        .iter()
        .filter(|f| !f.is_pk() && !f.is_ignored())
        .map(|f| &f.ident)
        .collect();

    let insert_cols = insert_fields
        .iter()
        .map(|id| id.to_string().to_lowercase())
        .collect::<Vec<_>>()
        .join(", ");

    let non_pk_cols = non_pk_fields
        .iter()
        .map(|id| id.to_string().to_lowercase())
        .collect::<Vec<_>>();

    let pk_col = pk_ident.to_string().to_lowercase();

    let insert_placeholders_str = {
        #[cfg(feature = "postgres")]
        {
            (1..=insert_fields.len())
                .map(|i| format!("${}", i))
                .collect::<Vec<_>>()
                .join(", ")
        }
        #[cfg(not(feature = "postgres"))]
        {
            vec!["?"; insert_fields.len()].join(", ")
        }
    };

    let update_set_clause = {
        #[cfg(feature = "postgres")]
        {
            non_pk_cols
                .iter()
                .enumerate()
                .map(|(i, name)| format!("{} = ${}", name, i + 1))
                .collect::<Vec<_>>()
                .join(", ")
        }
        #[cfg(not(feature = "postgres"))]
        {
            non_pk_cols
                .iter()
                .map(|name| format!("{} = ?", name))
                .collect::<Vec<_>>()
                .join(", ")
        }
    };

    let where_placeholder_str = {
        #[cfg(feature = "postgres")]
        {
            format!("${}", non_pk_fields.len() + 1)
        }
        #[cfg(not(feature = "postgres"))]
        {
            "?".to_string()
        }
    };

    let insert_sql = format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
        table_name, insert_cols, insert_placeholders_str
    );

    let update_sql = format!(
        "UPDATE {} SET {} WHERE {} = {} RETURNING *",
        table_name, update_set_clause, pk_col, where_placeholder_str
    );

    let created_assign = es
        .fields
        .iter()
        .find(|f| matches!(f.kind, FieldKind::Timestamp(TimestampKind::Created { .. })))
        .map(|f| {
            let ident = &f.ident;
            if let FieldKind::Timestamp(TimestampKind::Created { factory }) = &f.kind {
                quote! { self.#ident = #factory; }
            } else {
                quote! {}
            }
        })
        .unwrap_or_else(|| quote! {});

    let updated_assign_insert = es
        .fields
        .iter()
        .find(|f| matches!(f.kind, FieldKind::Timestamp(TimestampKind::Updated { .. })))
        .map(|f| {
            let ident = &f.ident;
            if let FieldKind::Timestamp(TimestampKind::Updated { factory }) = &f.kind {
                quote! { self.#ident = #factory; }
            } else {
                quote! {}
            }
        })
        .unwrap_or_else(|| quote! {});

    let updated_assign_update = updated_assign_insert.clone();

    let uuid_assigns = es
        .fields
        .iter()
        .filter(|f| !f.is_ignored() && is_uuid_type(&f.ty))
        .map(|f| {
            let ident = &f.ident;
            let ty = &f.ty;

            #[cfg(feature = "uuid")]
            quote! {
                if <#ty as Default>::default() == self.#ident {
                    self.#ident = uuid::Uuid::new_v4();
                }
            }
            #[cfg(not(feature = "uuid"))]
            quote! {}
        });

    quote! {
        #[automatically_derived]
        impl #s_ident {
            /// Inserts a new record into the database.
            ///
            /// This method forces an INSERT operation regardless of the primary key value.
            /// It automatically populates timestamp fields marked with:
            /// - `#[sql(timestamp(created_at, factory_fn()))]` - Set using custom factory
            /// - `#[sql(timestamp(updated_at, factory_fn()))]` - Set using custom factory
            ///
            /// Takes ownership of self and returns a new instance with the inserted record data,
            /// including any auto-generated primary key values.
            ///
            /// # Returns
            ///
            /// Returns `Ok(Self)` with the inserted record data, or an `sqlx::Error` if the
            /// insertion fails (e.g., due to constraint violations, database connection issues).
            ///
            /// # Example
            ///
            /// if the primary key is the default value , the record will be inserted, otherwise
            /// it will be updated.
            ///
            /// ```ignore
            /// let user = User {
            ///     email: "user@example.com".to_string(),
            ///     name: "John Doe".to_string(),
            ///     created_at: DateTime::default(), // Will be auto-populated
            ///     updated_at: DateTime::default(), // Will be auto-populated
            ///     ..Default::default() // Will be auto-populated
            /// };
            ///
            /// let inserted_user = user.insert(&pool).await?;
            /// println!("Inserted user with ID: {}", inserted_user.id);
            /// ```
            pub async fn insert<'a, E>(mut self, executor: E) -> sqlx::Result<Self>
            where
                E: ::sqlorm::sqlx::Executor<'a, Database = ::sqlorm::Driver>,
            {
                #(#uuid_assigns)*
                #created_assign
                #updated_assign_insert

                ::sqlorm::sqlx::query_as::<_, #s_ident>(#insert_sql)
                    #(.bind(&self.#insert_fields))*
                    .fetch_one(executor)
                    .await
            }

            /// Updates an existing record in the database.
            ///
            /// This method forces an UPDATE operation using the primary key to identify the record.
            /// It automatically updates timestamp fields marked with:
            /// - `#[sql(timestamp(updated_at, factory_fn()))]` - Set using custom factory
            ///
            /// Takes ownership of self and returns a new instance with the updated record data from the database.
            ///
            /// # Returns
            ///
            /// Returns `Ok(Self)` with the updated record data, or an `sqlx::Error` if the
            /// update fails (e.g., record not found, constraint violations, database connection issues).
            ///
            /// # Example
            ///
            /// ```ignore
            /// let mut user = User::find_by_id(&pool, 1).await?.expect("User not found");
            /// user.name = "Updated Name".to_string();
            ///
            /// let updated_user = user.update(&pool).await?;
            /// println!("Updated user: {}", updated_user.name);
            /// ```
            pub async fn update<'a, E>(
                mut self,
                executor: E
            ) -> ::sqlorm::sqlx::Result<Self>
            where
                E: ::sqlorm::sqlx::Executor<'a, Database = ::sqlorm::Driver>
            {
                #updated_assign_update

                ::sqlorm::sqlx::query_as::<_, #s_ident>(#update_sql)
                    #(.bind(&self.#non_pk_fields))*
                    .bind(&self.#pk_ident)
                    .fetch_one(executor)
                    .await
            }

            /// Saves the record to the database (insert if new, update if existing).
            ///
            /// This method automatically determines whether to perform an INSERT or UPDATE:
            /// - If the primary key equals the type's default value, performs an INSERT
            /// - Otherwise, performs an UPDATE
            ///
            /// This is the recommended method for most save operations as it handles both
            /// creation and modification scenarios automatically.
            ///
            /// # Returns
            ///
            /// Returns `Ok(Self)` with the saved record data, or an `sqlx::Error` if the
            /// operation fails.
            ///
            /// # Example
            ///
            /// ```rust ignore
            /// // New record (primary key is default/0)
            /// let new_user = User {
            ///     id: 0,
            ///     email: "new@example.com".to_string(),
            ///     name: "New User".to_string(),
            ///     created_at: DateTime::default(),
            ///     updated_at: DateTime::default(),
            /// };
            /// let saved = new_user.save(&pool).await?; // Will INSERT
            ///
            /// // Existing record (primary key is not default)
            /// let mut existing_user = User::query().fetch_one(&pool).await?.unwrap();
            /// existing_user.name = "Modified".to_string();
            /// let updated = existing_user.save(&pool).await?; // Will UPDATE
            /// ```
            pub async fn save<'a, E>(
                self,
                executor: E
            ) -> ::sqlorm::sqlx::Result<Self>
            where
                E: ::sqlorm::sqlx::Executor<'a, Database = ::sqlorm::Driver>
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
// user.update_fields(vec![&User::NAME, &User::LAST_NAME]).await
