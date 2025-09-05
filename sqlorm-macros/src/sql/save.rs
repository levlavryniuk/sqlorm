use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{
    entity::{EntityStruct, FieldKind, TimestampKind},
    sql::with_quotes,
};

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
    let field_count = non_pk_fields.len();
    let non_pk_names: Vec<String> = non_pk_fields
        .iter()
        .map(|id| id.to_string().to_lowercase())
        .collect();

    let insert_placeholders: Vec<String> = (1..=non_pk_fields.len())
        .map(|i| format!("${}", i))
        .collect();

    let update_assignments: Vec<String> = non_pk_names
        .iter()
        .enumerate()
        .map(|(i, name)| format!("{} = ${}", name, i + 1))
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
            pub async fn insert<'a, E>(
                &mut self,
                executor: E
            ) -> sqlx::Result<Self>
            where
                E: sqlx::PgExecutor<'a>
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
                    #(.bind(&self.#non_pk_fields))*;
                let saved = saved.fetch_one(executor)
                    .await?;

                *self = saved.clone();
                Ok(saved)
            }

            pub async fn update<'a, E>(
                &mut self,
                executor: E
            ) -> sqlx::Result<Self>
            where
                E: sqlx::PgExecutor<'a>
            {
                #updated_assign_update

                let query = format!(
                    r#"UPDATE {table}
                 SET {updates}
                 WHERE {pk} = ${pk_index}
                 RETURNING *"#,
                    table = #table_name,
                    updates = [#(#update_assignments),*].join(", "),
                    pk = #pk_name,
                    pk_index = #field_count + 1,
                );

                let saved = sqlx::query_as::<_, #s_ident>(&query)
                    #(.bind(&self.#non_pk_fields))*
                    .bind(&self.#pk_ident)
                    .fetch_one(executor)
                    .await?;

                *self = saved.clone();
                Ok(saved)
            }

            pub async fn save<'a, E>(
                &mut self,
                executor: E
            ) -> sqlx::Result<Self>
            where
                E: sqlx::PgExecutor<'a>
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
