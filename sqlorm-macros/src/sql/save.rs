//! Save operation generation for Entity macros.
//!
//! This module generates the `insert`, `update`, and `save` methods for entities,
//! handling automatic timestamp management and SQL generation for CRUD operations.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{
    entity::{EntityStruct, FieldKind, TimestampKind},
    sql::with_quotes,
};

/// Generates `insert`, `update`, and `save` method implementations for an entity.
///
/// Creates three methods:
/// - `insert(&mut self, executor)` - Forces an INSERT, auto-populates created_at/updated_at
/// - `update(&mut self, executor)` - Forces an UPDATE, auto-updates updated_at
/// - `save(&mut self, executor)` - INSERT if primary key is default, UPDATE otherwise
///
/// All methods:
/// - Use `RETURNING *` to get the complete updated record
/// - Update the struct instance with returned data
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

    let insert_placeholders: Vec<String> =
        (0..non_pk_fields.len()).map(|_| "?".to_string()).collect();

    let update_assignments: Vec<String> = non_pk_names
        .iter()
        .map(|name| format!("{} = ?", name))
        .collect();

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

    quote! {
        impl #s_ident {
            /// Inserts a new record into the database.
            ///
            /// This method forces an INSERT operation regardless of the primary key value.
            /// It automatically populates timestamp fields marked with:
            /// - `#[sql(timestamp = "created_at")]` - Set to current UTC time
            /// - `#[sql(timestamp = "updated_at")]` - Set to current UTC time
            ///
            /// The method updates the struct instance with the returned data from the database,
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
            /// ```rust
            /// let mut user = User {
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
            pub async fn insert<'a, E>(
                &mut self,
                executor: E
            ) -> sqlx::Result<Self>
            where
                E: sqlx::Executor<'a, Database = sqlorm::Driver>
            {
                #created_assign
                #updated_assign_insert

                let query = format!(
                    r#"insert into {table} ({cols})
                 values ({placeholders})
                 returning *"#,
                    table = #table_name,
                    cols = [#(#non_pk_names),*].join(", "),
                    placeholders = [#(#insert_placeholders),*].join(", "),
                );

                let saved = sqlx::query_as::<_, #s_ident>(&query)
                    #(.bind(&self.#non_pk_fields))*
                    .fetch_one(executor)
                    .await?;

                *self = saved.clone();
                Ok(saved)
            }

            /// Updates an existing record in the database.
            ///
            /// This method forces an UPDATE operation using the primary key to identify the record.
            /// It automatically updates timestamp fields marked with:
            /// - `#[sql(timestamp = "updated_at")]` - Set to current UTC time
            ///
            /// The method updates the struct instance with the returned data from the database.
            ///
            /// # Returns
            ///
            /// Returns `Ok(Self)` with the updated record data, or an `sqlx::Error` if the
            /// update fails (e.g., record not found, constraint violations, database connection issues).
            ///
            /// # Example
            ///
            /// ```rust
            /// let mut user = User::find_by_id(&pool, 1).await?.expect("User not found");
            /// user.name = "Updated Name".to_string();
            ///
            /// let updated_user = user.update(&pool).await?;
            /// println!("Updated user: {}", updated_user.name);
            /// ```
            pub async fn update<'a, E>(
                &mut self,
                executor: E
            ) -> sqlx::Result<Self>
            where
                E: sqlx::Executor<'a, Database = sqlorm::Driver>
            {
                #updated_assign_update

                let query = format!(
                    r#"UPDATE {table}
                 SET {updates}
                 WHERE {pk} = ?
                 RETURNING *"#,
                    table = #table_name,
                    updates = [#(#update_assignments),*].join(", "),
                    pk = #pk_name,
                );

                let saved = sqlx::query_as::<_, #s_ident>(&query)
                    #(.bind(&self.#non_pk_fields))*
                    .bind(&self.#pk_ident)
                    .fetch_one(executor)
                    .await?;

                *self = saved.clone();
                Ok(saved)
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
            /// ```rust
            /// // New record (primary key is default/0)
            /// let mut new_user = User {
            ///     id: 0,
            ///     email: "new@example.com".to_string(),
            ///     name: "New User".to_string(),
            ///     created_at: DateTime::default(),
            ///     updated_at: DateTime::default(),
            /// };
            /// let saved = new_user.save(&pool).await?; // Will INSERT
            ///
            /// // Existing record (primary key is not default)
            /// let mut existing_user = User::find_by_id(&pool, 1).await?.unwrap();
            /// existing_user.name = "Modified".to_string();
            /// let updated = existing_user.save(&pool).await?; // Will UPDATE
            /// ```
            pub async fn save<'a, E>(
                &mut self,
                executor: E
            ) -> sqlx::Result<Self>
            where
                E: sqlx::Executor<'a, Database = sqlorm::Driver>
            {
                if self.#pk_ident == #pk_type::default() {
                    self.insert(executor).await
                } else {
                    self.update(executor).await
                }
            }
        }
    }
}
