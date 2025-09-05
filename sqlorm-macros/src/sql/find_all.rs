use proc_macro2::TokenStream;
use quote::quote;

use crate::{entity::EntityStruct, sql::with_quotes};

pub fn find_all(es: &EntityStruct) -> TokenStream {
    let s_ident = &es.struct_ident;
    let table_name = with_quotes(&es.table_name);

    quote! {
        impl #s_ident {
            pub async fn find_all<'a, E>(
                executor: E
            ) -> sqlx::Result<Vec<Self>>
            where
                E: sqlx::PgExecutor<'a>
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
