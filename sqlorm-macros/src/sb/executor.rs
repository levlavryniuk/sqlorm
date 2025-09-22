// TODO - if operation fails, return old state of changed fields.
use crate::{
    EntityStruct,
    entity::{FieldKind, TimestampKind},
};
use quote::quote;
use sqlorm_core::with_quotes;

pub fn executor(es: &EntityStruct) -> proc_macro2::TokenStream {
    let ident = &es.struct_ident;
    let implementation = implementation(es);

    quote! {
        #[automatically_derived]
        #[::sqlorm::async_trait]
        impl ::sqlorm::StatementExecutor<#ident> for ::sqlorm::SB<#ident,::sqlorm::Update> {
            #implementation
        }
    }
}

pub fn implementation(es: &EntityStruct) -> proc_macro2::TokenStream {
    let table_name = with_quotes(&es.table_name.raw);
    let ident = &es.struct_ident;
    let pk_ident = &es.pk.ident;
    let pk_col = &es.pk.name;

    let updateable_fields: Vec<_> = es
        .fields
        .iter()
        .filter(|f| !f.is_pk() && !f.is_ignored())
        .collect();

    let all_columns: Vec<String> = updateable_fields.iter().map(|f| f.name.clone()).collect();

    let updated_assign_update = es
        .fields
        .iter()
        .find(|f| matches!(f.kind, FieldKind::Timestamp(TimestampKind::Updated { .. })))
        .map(|f| {
            let ident = &f.ident;
            if let FieldKind::Timestamp(TimestampKind::Updated { factory }) = &f.kind {
                quote! { self.entity.#ident = #factory; }
            } else {
                quote! {}
            }
        })
        .unwrap_or_else(|| quote! {});

    let placeholder_generator = if cfg!(feature = "postgres") {
        quote! {
            let placeholders: Vec<String> = (1..=fields_to_update.len())
                .map(|i| format!("${}", i))
                .collect();
            let where_placeholder = format!("${}", fields_to_update.len() + 1);
        }
    } else {
        quote! {
            let placeholders: Vec<String> = vec!["?".to_string(); fields_to_update.len()];
            let where_placeholder = "?".to_string();
        }
    };

    let field_bindings = updateable_fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_name = &field.name;
        quote! {
            #field_name => {
                query = query.bind(&self.entity.#field_ident);
            }
        }
    });

    quote! {
        async fn execute<'a, E>(
            mut self,
            acquirer: E
        ) -> ::sqlorm::sqlx::Result<#ident> where E: ::sqlorm::sqlx::Acquire<'a, Database = ::sqlorm::Driver> + Send{

            use ::sqlorm::sqlx::Acquire;
            let mut conn = acquirer.acquire().await?;

            #updated_assign_update

            let fallback_columns = vec![#(#all_columns),*];
            let fields_to_update = if let Some(f) = &self.fields {
                f
            } else {
                &fallback_columns
            };

            if fields_to_update.is_empty() {
                return Ok(self.entity);
            }

            // outputs `placeholders` and `where_placeholder` variables
            #placeholder_generator

            let set_clause: Vec<String> = fields_to_update
                .iter()
                .zip(&placeholders)
                .map(|(field, placeholder)| format!("{} = {}", field, placeholder))
                .collect();

            let sql = format!(
                "UPDATE {} SET {} WHERE {} = {}",
                #table_name,
                set_clause.join(", "),
                #pk_col,
                where_placeholder
            );
            let mut query = ::sqlorm::sqlx::query::<::sqlorm::Driver>(&sql);

            for field_name in fields_to_update {
                match field_name.as_ref() {
                    #(#field_bindings)*
                    _ => {}
                }
            }

            query = query.bind(&self.entity.#pk_ident);

            query.execute(&mut *conn).await?;

            Ok(self.entity)
        }
    }
}
