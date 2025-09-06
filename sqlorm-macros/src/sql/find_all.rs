use proc_macro2::TokenStream;
use quote::quote;

use crate::{entity::EntityStruct, sql::with_quotes};

pub fn find_all(es: &EntityStruct) -> TokenStream {
    let s_ident = &es.struct_ident;
    let table_name = with_quotes(&es.table_name);

    quote! {
        impl #s_ident {
            /// Retrieves all records from the database table.
            /// 
            /// This method executes a `SELECT * FROM table` query to fetch all records
            /// in the table. Use with caution on large tables as it loads all data into memory.
            /// 
            /// For tables with many records, consider using pagination or filtering methods
            /// provided by the query builder instead.
            /// 
            /// # Parameters
            /// 
            /// * `executor` - A database executor (connection, pool, or transaction)
            /// 
            /// # Returns
            /// 
            /// Returns `Ok(Vec<Self>)` containing all records, or an `sqlx::Error` if the
            /// query fails. An empty vector is returned if no records exist.
            /// 
            /// # Example
            /// 
            /// ```rust
            /// // Get all users
            /// let all_users = User::find_all(&pool).await?;
            /// println!("Found {} users", all_users.len());
            /// 
            /// for user in all_users {
            ///     println!("User: {} ({})", user.name, user.email);
            /// }
            /// 
            /// // Handle empty results
            /// if all_users.is_empty() {
            ///     println!("No users found in database");
            /// }
            /// ```
            /// 
            /// # Performance Note
            /// 
            /// This method loads all records into memory at once. For large datasets,
            /// consider using streaming queries or pagination techniques.
            pub async fn find_all<'a, E>(
                executor: E
            ) -> sqlx::Result<Vec<Self>>
            where
                E: sqlx::Executor<'a, Database = sqlorm_core::Driver>
            {
                let query = format!(
                    "SELECT * FROM {table}",
                    table = #table_name,
                );
                sqlx::query_as::<_, #s_ident>(&query)
                    .fetch_all(executor)
                    .await
            }
        }
    }
}
