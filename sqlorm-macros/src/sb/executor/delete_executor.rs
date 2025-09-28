use crate::{
    EntityStruct,
    entity::{FieldKind, TimestampKind},
};
use quote::quote;
use sqlorm_core::with_quotes;

pub fn executor(es: &EntityStruct) -> proc_macro2::TokenStream {
    let ident = &es.struct_ident;
    let implementation = delete_implementation(es);

    quote! {
        #[automatically_derived]
        #[::sqlorm::async_trait]
        impl ::sqlorm::StatementExecutor<#ident> for ::sqlorm::SB<#ident,::sqlorm::Delete> {
            #implementation
        }
    }
}

pub fn delete_implementation(es: &EntityStruct) -> proc_macro2::TokenStream {
    let table_name = with_quotes(&es.table_name.raw);
    let ident = &es.struct_ident;
    let pk_ident = &es.pk.ident;
    let pk_col = &es.pk.name;

    if let Some(f) = es
        .fields
        .iter()
        .find(|f| matches!(f.kind, FieldKind::Timestamp(TimestampKind::Deleted { .. })))
    {
        let deleted_at_col = &f.name;
        let deleted_at_ident = &f.ident;
        let factory = if let FieldKind::Timestamp(TimestampKind::Deleted { factory }) = &f.kind {
            factory
        } else {
            return syn::Error::new_spanned(
                deleted_at_ident,
                "You must provide factory for deleted_at timestamp: e.g. chrono::Utc::now()",
            )
            .to_compile_error();
        };

        let (placeholder1, placeholder2) = if cfg!(feature = "postgres") {
            ("$1", "$2")
        } else {
            ("?", "?")
        };

        quote! {
            async fn execute<'a, E>(
                mut self,
                acquirer: E
            ) -> ::sqlorm::sqlx::Result<#ident>
            where E: ::sqlorm::sqlx::Acquire<'a, Database = ::sqlorm::Driver> + Send
            {
                use ::sqlorm::sqlx::Acquire;
                let mut conn = acquirer.acquire().await?;
                let deleted_at = #factory;
                let sql = format!(
                    "UPDATE {} SET {} = {} WHERE {} = {}",
                    #table_name, #deleted_at_col, #placeholder1, #pk_col, #placeholder2
                );
                ::sqlorm::sqlx::query(&sql)
                    .bind(&deleted_at)
                    .bind(&self.entity.#pk_ident)
                    .execute(&mut *conn)
                    .await?;
                self.entity.#deleted_at_ident = Some(deleted_at);
                Ok(self.entity)
            }
        }
    } else {
        let placeholder = if cfg!(feature = "postgres") {
            "$1"
        } else {
            "?"
        };

        quote! {
            async fn execute<'a, E>(
                mut self,
                acquirer: E
            ) -> ::sqlorm::sqlx::Result<#ident>
            where E: ::sqlorm::sqlx::Acquire<'a, Database = ::sqlorm::Driver> + Send
            {
                use ::sqlorm::sqlx::Acquire;
                let mut conn = acquirer.acquire().await?;
                let sql = format!(
                    "DELETE FROM {} WHERE {} = {}",
                    #table_name, #pk_col, #placeholder
                );
                ::sqlorm::sqlx::query(&sql)
                    .bind(&self.entity.#pk_ident)
                    .execute(&mut *conn)
                    .await?;
                Ok(self.entity)
            }
        }
    }
}
