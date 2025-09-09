//! Save operation generation for Entity macros.
//!
//! This module generates the `insert`, `update`, and `save` methods for entities,
//! handling automatic timestamp management and SQL generation for CRUD operations.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

use crate::{
    entity::{EntityStruct, FieldKind, TimestampKind},
    sql::{generate_placeholders, generate_single_placeholder, with_quotes},
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
/// - Handle timestamp fields automatically based on their `#[sql(timestamp = "...")]` attributes
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
    let table_name = with_quotes(&es.table_name);

    let pk_field = &es.pk;
    let pk_ident = &pk_field.ident;

    let pk_name = pk_ident.to_string().to_lowercase();
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

    let insert_placeholders = generate_placeholders(insert_fields.len());

    let update_placeholders = generate_placeholders(non_pk_fields.len());
    let update_assignments: Vec<String> = non_pk_names
        .iter()
        .zip(update_placeholders.iter())
        .map(|(name, placeholder)| format!("{} = {}", name, placeholder))
        .collect();

    let where_placeholder = generate_single_placeholder(non_pk_fields.len() + 1);

    let created_assign = es
        .fields
        .iter()
        .find(|f| matches!(f.kind, FieldKind::Timestamp(TimestampKind::Created)))
        .map(|f| {
            let ident = &f.ident;
            quote! { self.#ident = chrono::Utc::now(); }
        });

    let updated_assign_insert = es
        .fields
        .iter()
        .find(|f| matches!(f.kind, FieldKind::Timestamp(TimestampKind::Updated)))
        .map(|f| {
            let ident = &f.ident;
            quote! { self.#ident = chrono::Utc::now(); }
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
        impl #s_ident {
            /// Inserts a new record into the database.
            ///
            /// This method forces an INSERT operation regardless of the primary key value.
            /// It automatically populates timestamp fields marked with:
            /// - `#[sql(timestamp = "created_at")]` - Set to current UTC time
            /// - `#[sql(timestamp = "updated_at")]` - Set to current UTC time
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

                let query_str = format!(
                    r#"INSERT INTO {table} ({cols})
                           VALUES ({placeholders})
                           RETURNING *"#,
                    table = #table_name,
                    cols = [#(#insert_names),*].join(", "),
                    placeholders = [#(#insert_placeholders),*].join(", "),
                );

                ::sqlorm::sqlx::query_as::<_, #s_ident>(&query_str)
                    #(.bind(&self.#insert_fields))*
                    .fetch_one(executor)
                    .await
            }

            /// Updates an existing record in the database.
            ///
            /// This method forces an UPDATE operation using the primary key to identify the record.
            /// It automatically updates timestamp fields marked with:
            /// - `#[sql(timestamp = "updated_at")]` - Set to current UTC time
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
            /// let user = User::find_by_id(&pool, 1).await?.expect("User not found");
            /// let mut user_to_update = user;
            /// user_to_update.name = "Updated Name".to_string();
            ///
            /// let updated_user = user_to_update.update(&pool).await?;
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

                let query = format!(
                    r#"UPDATE {table}
                           SET {updates}
                           WHERE {pk} = {where_clause}
                           RETURNING *"#,
                    table = #table_name,
                    updates = [#(#update_assignments),*].join(", "),
                    pk = #pk_name,
                    where_clause = #where_placeholder,
                );

                ::sqlorm::sqlx::query_as::<_, #s_ident>(&query)
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
            /// ```ignore
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
            /// let existing_user = User::find_by_id(&pool, 1).await?.unwrap();
            /// let mut user_to_save = existing_user;
            /// user_to_save.name = "Modified".to_string();
            /// let updated = user_to_save.save(&pool).await?; // Will UPDATE
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
