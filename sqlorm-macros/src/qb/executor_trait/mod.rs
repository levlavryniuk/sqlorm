use proc_macro2::TokenStream;
use syn::Ident;

use crate::{naming::executor_from_entity_ident, relations::RelationType};

#[derive(PartialEq)]
pub enum FetchVariant {
    Eager,
    Batch,
}
impl From<&RelationType> for FetchVariant {
    fn from(rt: &RelationType) -> Self {
        match rt {
            RelationType::BelongsTo | RelationType::HasOne => FetchVariant::Eager,
            RelationType::HasMany => FetchVariant::Batch,
        }
    }
}

pub fn executor_trait(es: &crate::EntityStruct) -> proc_macro2::TokenStream {
    let s_name = &es.struct_ident;
    let tident = executor_from_entity_ident(&es.struct_ident);
    let eager: Vec<TokenStream> = es
        .relations
        .iter()
        .filter_map(|r| {
            if FetchVariant::from(&r.kind) != FetchVariant::Eager {
                return None;
            }
            let r_name = &r.relation_name;
            let other = &r.other;
            let on = Ident::new(r_name, other.span());
            Some(quote::quote! {
                if let Some(relation) = self.eager.iter().find(|rel| rel.relation_name == #r_name) {
                    let related_entity: #other = sqlorm::core::FromAliasedRow::from_aliased_row(&row)?;
                    core.#on = Some(related_entity);
                }
            })
        })
        .collect();

    let batch_one: Vec<TokenStream> = es
        .relations
        .iter()
        .filter_map(|r| {
            if FetchVariant::from(&r.kind) != FetchVariant::Batch {
                return None;
            }
            let r_name = &r.relation_name;
            let other = &r.other;
            let on = Ident::new(r_name, other.span());
            let (parent_key, foreign_key) = (&r.on.0, &r.on.1);

            let foreign_key_const =
                Ident::new(&foreign_key.to_string().to_uppercase(), foreign_key.span());

            Some(quote::quote! {
                if let Some(relation) = self.batch.iter().find(|rel| rel.relation_name == #r_name) {
                    let parent_id = core.#parent_key;

                    let children: Vec<#other> = #other::query()
                        .filter(#other::#foreign_key_const.eq(parent_id.clone()))
                        .fetch_all(pool)
                        .await?;

                    core.#on = Some(children);
                }
            })
        })
        .collect();

    let batch_all: Vec<TokenStream> = es
        .relations
        .iter()
        .filter_map(|r| {
            if FetchVariant::from(&r.kind) != FetchVariant::Batch {
                return None;
            }
            let r_name = &r.relation_name;
            let other = &r.other;
            let on = Ident::new(r_name, other.span());
            let (parent_key, foreign_key) = (&r.on.0, &r.on.1);

            let foreign_key_const =
                Ident::new(&foreign_key.to_string().to_uppercase(), foreign_key.span());

            Some(quote::quote! {
                if let Some(relation) = self.batch.iter().find(|rel| rel.relation_name == #r_name) {
                    let parent_ids: Vec<_> = results.iter().map(|p| p.#parent_key).collect();

                    if !parent_ids.is_empty() {
                        let related: Vec<#other> = #other::query()
                            .filter(#other::#foreign_key_const.in_(parent_ids.clone()))
                            .fetch_all(pool)
                            .await?;

                        use std::collections::HashMap;
                        let mut grouped: HashMap<_, Vec<#other>> = HashMap::new();
                        for rel in related {
                            let key = rel.#foreign_key;
                            grouped.entry(key).or_default().push(rel);
                        }

                        for parent in &mut results {
                            if let Some(children) = grouped.remove(&parent.#parent_key) {
                                parent.#on = Some(children);
                            } else {
                                parent.#on = Some(Vec::new());
                            }
                        }
                    }
                }
            })
        })
        .collect();

    quote::quote! {
        #[sqlorm::core::async_trait]
        pub trait #tident
        where
            #s_name: Send + Sync + sqlorm::core::Table + 'static,
        {
            async fn fetch_one(self, pool: &sqlorm::core::Pool) -> sqlx::Result<#s_name>;
            async fn fetch_optional(self, pool: &sqlorm::core::Pool) -> sqlx::Result<Option<#s_name>>;
            async fn fetch_all(self, pool: &sqlorm::core::Pool) -> sqlx::Result<Vec<#s_name>>;
        }

        #[sqlorm::core::async_trait]
        impl #tident for sqlorm::core::QB<#s_name> where
    #s_name: Send + Sync + sqlorm::core::Table + 'static,{
            async fn fetch_one(self, pool: &sqlorm::core::Pool) -> sqlx::Result<#s_name> {
                if self.eager.is_empty() && self.batch.is_empty() {
                    let row = self.build_query().build().fetch_one(pool).await?;
                    let core:#s_name = sqlorm::core::FromAliasedRow::from_aliased_row(&row)?;
                    return Ok(core);
                }

                let row = self.build_query().build().fetch_one(pool).await?;
                let mut core:#s_name = sqlorm::core::FromAliasedRow::from_aliased_row(&row)?;

                #(#eager)*
                #(#batch_one)*

                Ok(core)
            }

            async fn fetch_optional(self, pool: &sqlorm::core::Pool) -> sqlx::Result<Option<#s_name>> {
                if self.eager.is_empty() && self.batch.is_empty() {
                    let row = self.build_query().build().fetch_optional(pool).await?;
                    if let Some(row) = row {
                        let core:#s_name = sqlorm::core::FromAliasedRow::from_aliased_row(&row)?;
                        return Ok(Some(core));
                    }
                    return Ok(None);
                }

                let row = self.build_query().build().fetch_optional(pool).await?;
                if let Some(row) = row {
                    let mut core:#s_name = sqlorm::core::FromAliasedRow::from_aliased_row(&row)?;

                    #(#eager)*
                    #(#batch_one)*

                    Ok(Some(core))
                } else {
                    Ok(None)
                }
            }

            async fn fetch_all(self, pool: &sqlorm::core::Pool) -> sqlx::Result<Vec<#s_name>> {
                let rows = self.build_query().build().fetch_all(pool).await?;
                let mut results = Vec::new();

                for row in rows {
                    let mut core: #s_name = sqlorm::core::FromAliasedRow::from_aliased_row(&row)?;
                    #(#eager)*
                    results.push(core);
                }

                #(#batch_all)*

                Ok(results)
            }
        }
    }
}
