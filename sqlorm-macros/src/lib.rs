use proc_macro::TokenStream;
use syn::parse_macro_input;

// Note: proc-macro crates cannot export items, so we use full paths in generated code

mod naming;
mod traits;

use crate::entity::EntityStruct;
mod entity;
mod qb;
mod sql;

mod attrs;
mod gen_columns;
mod relations;

/// Derives the `Entity` trait and generates database interaction methods for a struct.
/// 
/// This macro transforms a regular Rust struct into a database entity by generating:
/// - SQL query methods (`find_by_*` for unique fields, `find_all`)
/// - CRUD operations (`insert`, `update`, `save`)
/// - Query builder integration
/// - Relationship handling (lazy loading)
/// - Column enumeration for type-safe queries
/// 
/// # Attributes
/// 
/// ## Table Configuration
/// - `#[table_name(name = "custom_table")]` - Override the default table name (defaults to struct name + "s")
/// 
/// ## Field Attributes (using `#[sql(...)]`)
/// - `#[sql(pk)]` - Mark field as the primary key (exactly one required)
/// - `#[sql(unique)]` - Mark field as unique (generates `find_by_*` methods)
/// - `#[sql(timestamp = "created_at")]` - Auto-populated timestamp on insert
/// - `#[sql(timestamp = "updated_at")]` - Auto-updated timestamp on insert/update
/// - `#[sql(timestamp = "deleted_at")]` - Soft delete timestamp field
/// - `#[sql(relation(...))]` - Define relationships (see below)
/// 
/// ## Relationship Syntax
/// ```text
/// #[sql(relation(TYPE -> TargetEntity, relation = "field_name", on = foreign_key_field))]
/// ```
/// Where TYPE is one of: `belongs_to`, `has_many`, `has_one`
/// 
/// ## SQLx Integration
/// - `#[sqlx(skip)]` - Exclude field from SQL operations (for computed/relationship fields)
/// 
/// # Example
/// 
/// ```rust
/// use sqlorm::Entity;
/// use chrono::{DateTime, Utc};
/// use serde::{Serialize, Deserialize};
/// 
/// #[derive(Debug, Entity, sqlx::FromRow, Clone, Serialize, Deserialize)]
/// #[table_name(name = "users")]
/// pub struct User {
///     #[sql(pk)]
///     #[sql(relation(has_many -> Post, relation = "posts", on = user_id))]
///     pub id: i64,
///     
///     #[sql(unique)]
///     pub email: String,
///     
///     pub name: String,
///     
///     #[sqlx(skip)]
///     pub posts: Option<Vec<Post>>,
///     
///     #[sql(timestamp = "created_at")]
///     pub created_at: DateTime<Utc>,
///     
///     #[sql(timestamp = "updated_at")]
///     pub updated_at: DateTime<Utc>,
/// }
/// ```
/// 
/// This generates methods like:
/// - `User::find_by_id(executor, id)` - Find by primary key
/// - `User::find_by_email(executor, email)` - Find by unique field
/// - `User::find_all(executor)` - Get all records
/// - `user.save(executor)` - Insert (if new) or update (if existing)
/// - `user.insert(executor)` - Force insert with auto-timestamps
/// - `user.update(executor)` - Force update with auto-timestamps
/// - `user.posts(executor).await` - Load related posts
#[proc_macro_derive(Entity, attributes(sql, table_name))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let es = parse_macro_input!(input as EntityStruct);
    entity::handle(es).into()
}

/// Generates column enumeration for query builder integration.
/// 
/// This macro creates a `Columns` enum for the entity struct, enabling type-safe
/// column references in query building. Each non-ignored field becomes a variant
/// in the enum.
/// 
/// The generated enum implements traits necessary for the query builder system,
/// allowing for compile-time verified SQL generation.
/// 
/// # Note
/// 
/// This macro is typically used internally by the `Entity` derive macro and is not
/// commonly used directly by end users. It's automatically applied when deriving `Entity`.
#[proc_macro_derive(GenColumns)]
pub fn gen_columns_handler(input: TokenStream) -> TokenStream {
    let es = parse_macro_input!(input as EntityStruct);
    gen_columns::handle(&es).into()
}
