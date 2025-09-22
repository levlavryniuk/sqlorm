use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlorm::prelude::*;
use sqlorm::table;
use uuid::Uuid;

#[table]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    #[sql(pk)]
    #[sql(relation(has_many -> Jar, relation = "jars", on = owner_id))]
    #[sql(relation(has_many -> Donation, relation = "payed_donations", on = payer_id))]
    pub id: i64,
    #[sql(unique)]
    pub email: String,
    #[serde(skip)]
    #[allow(dead_code)]
    pub password: String,
    #[sql(unique)]
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub wallpaper_url: Option<String>,
    #[sql(rename("avatar"))]
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    #[sql(timestamp(created_at, chrono::Utc::now()))]
    pub created_at: DateTime<Utc>,
    #[sql(timestamp(updated_at, chrono::Utc::now()))]
    pub updated_at: DateTime<Utc>,
    #[sql(timestamp(deleted_at, chrono::Utc::now()))]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[table]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Jar {
    #[sql(pk)]
    #[sql(relation(has_many -> Donation, relation = "donations", on = jar_id))]
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub minimal_donation: f64,
    pub total_amount: f64,
    pub total_donations: i32,
    #[sql(unique)]
    pub alias: String,
    pub hide_earnings: bool,
    pub goal: Option<f64>,
    #[sql(relation(belongs_to -> User, relation = "owner", on = id))]
    pub owner_id: i64,
    #[sql(timestamp(created_at, chrono::Utc::now()))]
    pub created_at: DateTime<Utc>,
    #[sql(timestamp(updated_at, chrono::Utc::now()))]
    pub updated_at: DateTime<Utc>,
    #[sql(timestamp(deleted_at, chrono::Utc::now()))]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[table]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Donation {
    #[sql(pk)]
    pub id: Uuid,
    pub amount: f64,
    pub tip: f64,
    #[sql(relation(belongs_to -> Jar, relation = "jar", on = id))]
    pub jar_id: i64,
    #[sql(relation(belongs_to -> User, relation = "payer", on = id))]
    pub payer_id: i64,
    pub is_payed: bool,
    pub transaction_id: Option<String>,
    pub note: Option<String>,
    pub is_refunded: bool,
    pub refunded_at: Option<DateTime<Utc>>,
    #[sql(timestamp(deleted_at, chrono::Utc::now()))]
    pub deleted_at: Option<DateTime<Utc>>,
    #[sql(timestamp(created_at, chrono::Utc::now()))]
    pub created_at: DateTime<Utc>,
    #[sql(timestamp(updated_at, chrono::Utc::now()))]
    pub updated_at: DateTime<Utc>,
    pub payed_at: Option<DateTime<Utc>>,
}

impl User {
    /// Creates a test user with default values
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
