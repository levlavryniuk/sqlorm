use proc_macro::TokenStream;
use syn::ItemStruct;
use syn::parse_macro_input;

mod naming;
mod traits;

use crate::entity::EntityStruct;
mod entity;
mod qb;
mod sql;

mod attrs;
mod gen_columns;
mod relations;

#[proc_macro_derive(Entity, attributes(sql))]
pub fn entity(input: TokenStream) -> TokenStream {
    let es = parse_macro_input!(input as EntityStruct);
    entity::handle(es).into()
}

/// Transforms a struct into a database entity with ORM capabilities.
///
/// This is the primary way to define database entities in SQLOrm. The macro automatically
/// generates all necessary database operations like `save()`, `find_by_id()`, and query methods.
///
/// # Basic Usage
///
/// ```rust,ignore
/// use sqlorm::table;
///
/// #[table]
/// struct User {
///     #[sql(pk)]
///     id: i64,
///     email: String,
///     name: String,
/// }
/// ```
///
/// # Custom Table Name
///
/// ```rust,ignore
/// #[table(name = "app_users")]
/// struct User {
///     #[sql(pk)]
///     id: i64,
///     email: String,
/// }
/// ```
///
/// After applying this macro, you can use standard ORM operations:
/// - `user.save(&pool).await`
/// - `User::query().filter(...).fetch_all(&pool).await`
///
/// With feature `extra-traits` enable
/// - `User::find_by_id(&pool, 1).await`
/// - `User::find_by_email(&pool, "user@example.com".to_string()).await`
///
/// # Supported Field Attributes
///
/// ## `#[sql(...)]` Attributes
///
/// - **`pk`** - Mark field as primary key (required, exactly one per struct)
/// - **`unique`** - Mark field as unique (generates `find_by_*` methods)
/// - **`skip`** - Exclude field from SQL operations
/// - **`timestamp(field_name, factory)`** - Automatic timestamp management:
///   - `created_at` - Set on insert
///   - `updated_at` - Set on insert and update  
///   - `deleted_at` - For soft deletes
/// - **`relation(...)`** - Define relationships:
///   - `belongs_to -> SomeOtherStruct, relation = "some_other_struct", on = field`
///   - `has_many -> SomeOtherStruct, relation = "some_other_structs", on = field`
///   - `has_one -> SomeOtherStruct, relation = "some_other_struct", on = field`
///
/// ## `#[sqlx(...)]` Attributes
///
/// - **`skip`** - Exclude field from sqlx FromRow deserialization
///
/// # Complete Example
///
/// ```rust,ignore
/// use chrono::{DateTime, Utc};
/// use sqlorm::table;
///
/// #[derive(Debug, Clone, Default)]
/// #[table(name = "users")]
/// pub struct User {
///     #[sql(pk)]
///     #[sql(relation(has_many -> Post, relation = "posts", on = user_id))]
///     pub id: i64,
///     
///     #[sql(unique)]
///     pub email: String,
///     
///     #[sql(unique)]
///     pub username: String,
///     
///     pub first_name: String,
///     pub last_name: String,
///     
///     #[sql(skip)]
///     #[sqlx(skip)]
///     pub posts: Option<Vec<Post>>,
///     
///     #[sql(timestamp(created_at, chrono::Utc::now()))]
///     pub created_at: DateTime<Utc>,
///     
///     #[sql(timestamp(updated_at, chrono::Utc::now()))]
///     pub updated_at: DateTime<Utc>,
/// }
///
/// #[derive(Debug, Clone, Default)]
/// #[table(name = "posts")]
/// pub struct Post {
///     #[sql(pk)]
///     pub id: i64,
///     
///     pub title: String,
///     pub content: String,
///     
///     #[sql(relation(belongs_to -> User, relation = "author", on = id))]
///     pub user_id: i64,
///     
///     #[sql(skip)]
///     #[sqlx(skip)]
///     pub author: Option<User>,
/// }
/// ```
///
#[proc_macro_attribute]
pub fn table(args: TokenStream, input: TokenStream) -> TokenStream {
    let model = parse_macro_input!(input as ItemStruct);

    let table_name = if args.is_empty() {
        model.ident.to_string().to_lowercase()
    } else {
        let meta_list: syn::punctuated::Punctuated<syn::MetaNameValue, syn::Token![,]> =
            syn::parse_macro_input!(args with syn::punctuated::Punctuated::parse_terminated);

        let mut table_name = model.ident.to_string().to_lowercase();
        for meta in meta_list {
            if meta.path.is_ident("name") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = meta.value
                {
                    table_name = lit_str.value();
                    break;
                }
            }
        }
        table_name
    };

    quote::quote! {
        #[derive(::sqlorm::sqlx::FromRow,::sqlorm::Entity)]
        #[sql(name = #table_name)]
        #model
    }
    .into()
}
