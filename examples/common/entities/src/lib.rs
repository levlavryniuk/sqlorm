use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlorm::Entity;
use uuid::Uuid;

/// User entity with unique email field and timestamp tracking.
/// Demonstrates:
/// - Primary key with `#[sql(pk)]`
/// - Unique field with `#[sql(unique)]` (generates `find_by_email` method)
/// - Automatic timestamp management for created_at/updated_at
/// - Has-many relationship to Jar entities
/// - Skipped field for lazy-loaded relationships
#[derive(Debug, Entity, sqlx::FromRow, Clone, Default, Serialize, Deserialize)]
#[table_name(name = "users")]
pub struct User {
    #[sql(pk)]
    #[sql(relation(has_many -> Jar, relation = "jars", on = owner_id))]
    #[sql(relation(has_many -> Donation, relation = "payed_donations", on = payer_id))]
    pub id: i64,
    
    #[sql(unique)]
    pub email: String,
    
    pub password: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub bio: Option<String>,
    pub wallpaper_url: Option<String>,
    
    #[sqlx(skip)]
    pub jars: Option<Vec<Jar>>,
    
    #[sqlx(skip)]
    pub payed_donations: Option<Vec<Donation>>,
    
    #[sql(timestamp = "created_at")]
    pub created_at: DateTime<Utc>,
    
    #[sql(timestamp = "updated_at")]
    pub updated_at: DateTime<Utc>,
}

/// Jar entity representing a donation jar.
/// Demonstrates:
/// - Primary key with auto-increment
/// - Belongs-to relationship to User via owner_id
/// - Has-many relationship to Donation entities
/// - Optional fields with proper nullability
#[derive(Debug, Entity, sqlx::FromRow, Clone, Default, Serialize, Deserialize)]
#[table_name(name = "jars")]
pub struct Jar {
    #[sql(pk)]
    #[sql(relation(belongs_to -> User, relation = "owner", on = owner_id))]
    #[sql(relation(has_many -> Donation, relation = "donations", on = jar_id))]
    pub id: i64,
    
    pub title: String,
    pub description: Option<String>,
    pub minimal_donation: f64,
    pub total_amount: f64,
    pub total_donations: i32,
    
    #[sql(unique)]
    pub alias: String,
    
    pub goal: Option<f64>,
    
    /// Foreign key to users table
    pub owner_id: i64,
    
    #[sqlx(skip)]
    pub owner: Option<User>,
    
    #[sqlx(skip)]
    pub donations: Option<Vec<Donation>>,
    
    #[sql(timestamp = "created_at")]
    pub created_at: DateTime<Utc>,
    
    #[sql(timestamp = "updated_at")]
    pub updated_at: DateTime<Utc>,
}

/// Donation entity with UUID primary key.
/// Demonstrates:
/// - UUID primary key instead of integer
/// - Multiple belongs-to relationships
/// - Boolean fields
/// - Optional timestamp fields with nullable handling
#[derive(Debug, Entity, sqlx::FromRow, Clone, Serialize, Deserialize)]
#[table_name(name = "donations")]
pub struct Donation {
    #[sql(pk)]
    #[sql(relation(belongs_to -> Jar, relation = "jar", on = jar_id))]
    #[sql(relation(belongs_to -> User, relation = "payer", on = payer_id))]
    pub id: Uuid,
    
    pub amount: f64,
    pub tip: f64,
    pub transaction_id: Option<String>,
    pub note: Option<String>,
    pub is_payed: bool,
    pub is_refunded: bool,
    
    /// Foreign keys
    pub jar_id: i64,
    pub payer_id: i64,
    
    #[sqlx(skip)]
    pub jar: Option<Jar>,
    
    #[sqlx(skip)]
    pub payer: Option<User>,
    
    pub payed_at: Option<DateTime<Utc>>,
    pub refunded_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    
    #[sql(timestamp = "created_at")]
    pub created_at: DateTime<Utc>,
    
    #[sql(timestamp = "updated_at")]
    pub updated_at: DateTime<Utc>,
}

impl Default for Donation {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            amount: 0.0,
            tip: 0.0,
            transaction_id: None,
            note: None,
            is_payed: false,
            is_refunded: false,
            jar_id: 0,
            payer_id: 0,
            jar: None,
            payer: None,
            payed_at: None,
            refunded_at: None,
            deleted_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// Helper function to create test data
impl User {
    /// Creates a test user with default values
    pub fn test_user(email: &str, username: &str) -> Self {
        Self {
            email: email.to_string(),
            username: username.to_string(),
            password: "secret".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            ..Default::default()
        }
    }
}

impl Jar {
    /// Creates a test jar with default values
    pub fn test_jar(owner_id: i64, alias: &str) -> Self {
        Self {
            title: "Test Jar".to_string(),
            description: Some("A test jar".to_string()),
            minimal_donation: 1.0,
            alias: alias.to_string(),
            owner_id,
            ..Default::default()
        }
    }
}

impl Donation {
    /// Creates a test donation with default values
    pub fn test_donation(jar_id: i64, payer_id: i64, amount: f64) -> Self {
        Self {
            amount,
            tip: amount * 0.1,
            jar_id,
            payer_id,
            is_payed: true,
            ..Default::default()
        }
    }
}
