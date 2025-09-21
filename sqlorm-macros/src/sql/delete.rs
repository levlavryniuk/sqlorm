use sqlorm_core::with_quotes;

use crate::{
    EntityStruct,
    entity::{FieldKind, TimestampKind},
};

pub fn delete(es: &EntityStruct) -> proc_macro2::TokenStream {
    let table_name = with_quotes(&es.table_name.raw);
    let ident = &es.struct_ident;
    let pk_ident = &es.pk.ident;
    let pk_field = &es.pk.ident.to_string();
    let (placeholder, placeholder2) = if cfg!(feature = "postgres") {
        ("$1", "$2")
    } else {
        ("?", "?")
    };
    let sql = if let Some(f) = &es
        .fields
        .iter()
        .find(|f| matches!(f.kind, FieldKind::Timestamp(TimestampKind::Deleted { .. })))
    {
        let deleted_at_col = &f.ident.to_string();
        let deleted_at_ident = &f.ident;
        let factory = if let FieldKind::Timestamp(TimestampKind::Deleted { factory }) = &f.kind {
            factory
        } else {
            return syn::Error::new_spanned(
                deleted_at_ident,
                "You must provide factory for timestamp; Example: chrono::Utc::now()",
            )
            .to_compile_error();
        };
        quote::quote! {
            let deleted_at =  #factory;
            let sql = format!("update {} set {} = {} where {} = {}",
                #table_name,#deleted_at_col,#placeholder,#pk_field,#placeholder2
            );
            ::sqlorm::sqlx::query(&sql).bind(&deleted_at).bind(self.#pk_ident).execute(&mut *conn).await?;
            self.#deleted_at_ident=Some(deleted_at);
        }
    } else {
        quote::quote! {
            let sql = format!(
                "delete from {} where {} = {}",
                #table_name, #pk_field, #placeholder
            );
            ::sqlorm::sqlx::query(&sql).bind(self.#pk_ident).execute(&mut *conn).await?;
        }
    };

    quote::quote! {
        #[automatically_derived]
        impl #ident {
            /// ## Deletes row by primary key
            /// Performs soft delete if Entity has timestamp of kind `deleted_at`,
            /// otherwise, hard delete.
            pub async fn delete<'a,A>(mut self,acq: A) -> ::sqlorm::sqlx::Result<()>
            where A: ::sqlorm::sqlx::Acquire<'a,Database=::sqlorm::Driver>
            {
                let mut conn  = acq.acquire().await?;
                #sql
                Ok(())
            }
        }
    }
}
