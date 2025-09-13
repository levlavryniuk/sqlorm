mod common;

use chrono::{DateTime, NaiveDateTime, Utc};
use common::create_clean_db;
use sqlorm::table;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default)]
#[table(name = "chrono_entities")]
pub struct ChronoEntity {
    #[sql(pk)]
    pub id: i64,
    pub name: String,

    #[sql(timestamp(created_at, chrono::Utc::now()))]
    pub created_at: DateTime<Utc>,

    #[sql(timestamp(updated_at, chrono::Utc::now()))]
    pub updated_at: DateTime<Utc>,

    #[sql(timestamp(deleted_at, chrono::Utc::now()))]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
#[table(name = "naive_datetime_entities")]
pub struct NaiveDateTimeEntity {
    #[sql(pk)]
    pub id: i64,
    pub name: String,

    #[sql(timestamp(created_at, chrono::Utc::now().naive_utc()))]
    pub created_at: NaiveDateTime,

    #[sql(timestamp(updated_at, chrono::Utc::now().naive_utc()))]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Default)]
#[table(name = "custom_timestamp_entities")]
pub struct CustomTimestampEntity {
    #[sql(pk)]
    pub id: i64,
    pub name: String,

    #[sql(timestamp(created_at, get_custom_timestamp()))]
    pub created_at: i64,

    #[sql(timestamp(updated_at, get_custom_timestamp()))]
    pub updated_at: i64,
}

fn get_custom_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64
}

#[tokio::test]
async fn test_chrono_utc_timestamps() {
    let pool = create_clean_db().await;

    let before_insert = Utc::now();

    let entity = ChronoEntity {
        name: "test_chrono".to_string(),
        ..Default::default()
    };

    let saved_entity = entity
        .save(&pool)
        .await
        .expect("Failed to save chrono entity");

    let after_insert = Utc::now();

    assert!(saved_entity.id > 0);
    assert_eq!(saved_entity.name, "test_chrono");

    assert!(saved_entity.created_at >= before_insert);
    assert!(saved_entity.created_at <= after_insert);
    assert!(saved_entity.updated_at >= before_insert);
    assert!(saved_entity.updated_at <= after_insert);

    let time_diff = (saved_entity.updated_at - saved_entity.created_at)
        .num_milliseconds()
        .abs();
    assert!(
        time_diff < 1000,
        "created_at and updated_at should be very close on insert"
    );

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let mut updated_entity = saved_entity.clone();
    updated_entity.name = "updated_chrono".to_string();

    let before_update = Utc::now();
    let updated_entity = updated_entity
        .save(&pool)
        .await
        .expect("Failed to update chrono entity");
    let after_update = Utc::now();

    assert_eq!(updated_entity.created_at, saved_entity.created_at);
    assert!(updated_entity.updated_at >= before_update);
    assert!(updated_entity.updated_at <= after_update);
    assert!(updated_entity.updated_at > saved_entity.updated_at);
}

#[tokio::test]
async fn test_naive_datetime_timestamps() {
    let pool = create_clean_db().await;

    let before_insert = Utc::now().naive_utc();

    let entity = NaiveDateTimeEntity {
        name: "test_naive_datetime".to_string(),
        ..Default::default()
    };

    let saved_entity = entity
        .save(&pool)
        .await
        .expect("Failed to save naive datetime entity");

    let after_insert = Utc::now().naive_utc();

    assert!(saved_entity.id > 0);
    assert_eq!(saved_entity.name, "test_naive_datetime");

    assert!(saved_entity.created_at >= before_insert);
    assert!(saved_entity.created_at <= after_insert);
    assert!(saved_entity.updated_at >= before_insert);
    assert!(saved_entity.updated_at <= after_insert);

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let mut updated_entity = saved_entity.clone();
    updated_entity.name = "updated_naive_datetime".to_string();

    let before_update = Utc::now().naive_utc();
    let updated_entity = updated_entity
        .save(&pool)
        .await
        .expect("Failed to update naive datetime entity");

    assert_eq!(updated_entity.created_at, saved_entity.created_at);
    assert!(updated_entity.updated_at >= before_update);
    assert!(updated_entity.updated_at > saved_entity.updated_at);
}

#[tokio::test]
async fn test_custom_timestamp_factory() {
    let pool = create_clean_db().await;

    let before_insert = get_custom_timestamp();

    let entity = CustomTimestampEntity {
        name: "test_custom".to_string(),
        ..Default::default()
    };

    let saved_entity = entity
        .save(&pool)
        .await
        .expect("Failed to save custom timestamp entity");

    let after_insert = get_custom_timestamp();

    assert!(saved_entity.id > 0);
    assert_eq!(saved_entity.name, "test_custom");

    assert!(saved_entity.created_at >= before_insert);
    assert!(saved_entity.created_at <= after_insert);
    assert!(saved_entity.updated_at >= before_insert);
    assert!(saved_entity.updated_at <= after_insert);

    assert!(saved_entity.created_at > 1577836800); // 2020-01-01
    assert!(saved_entity.updated_at > 1577836800); // 2020-01-01

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let mut updated_entity = saved_entity.clone();
    updated_entity.name = "updated_custom".to_string();

    let before_update = get_custom_timestamp();
    let updated_entity = updated_entity
        .save(&pool)
        .await
        .expect("Failed to update custom timestamp entity");

    assert_eq!(updated_entity.created_at, saved_entity.created_at);
    assert!(updated_entity.updated_at >= before_update);
    assert!(updated_entity.updated_at >= saved_entity.updated_at);
}

#[tokio::test]
async fn test_deleted_at_timestamp() {
    let pool = create_clean_db().await;

    let entity = ChronoEntity {
        name: "test_soft_delete".to_string(),
        ..Default::default()
    };

    let saved_entity = entity.save(&pool).await.expect("Failed to save entity");

    assert!(saved_entity.deleted_at.is_none());

    let mut soft_deleted = saved_entity.clone();
    soft_deleted.deleted_at = Some(Utc::now());

    let updated_entity = soft_deleted
        .save(&pool)
        .await
        .expect("Failed to update entity with deleted_at");

    assert!(updated_entity.deleted_at.is_some());
    assert!(updated_entity.deleted_at.expect("deleted_at should be set") > saved_entity.created_at);
}

#[tokio::test]
async fn test_timestamp_update_behavior() {
    let pool = create_clean_db().await;

    let entity = ChronoEntity {
        name: "test_update_behavior".to_string(),
        ..Default::default()
    };

    let saved_entity = entity.save(&pool).await.expect("Failed to save entity");

    let original_created_at = saved_entity.created_at;
    let original_updated_at = saved_entity.updated_at;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let mut updated_entity = saved_entity.clone();
    updated_entity.name = "updated_name".to_string();

    let final_entity = updated_entity
        .save(&pool)
        .await
        .expect("Failed to update entity");

    assert_eq!(
        final_entity.created_at, original_created_at,
        "created_at should not change on update"
    );
    assert!(
        final_entity.updated_at > original_updated_at,
        "updated_at should be newer after update"
    );
    assert_eq!(final_entity.name, "updated_name");
}
