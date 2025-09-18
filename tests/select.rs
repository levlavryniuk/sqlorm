mod common;

use common::create_clean_db;
use common::entities::{Donation, Jar, User};
use sqlorm::GenericExecutor;
use uuid::Uuid;

async fn setup_select_test_data(pool: &sqlorm::Pool) -> (User, Jar, Donation) {
    let mut user = User::test_user("select@example.com", "selectuser");
    user.bio = Some("A test bio".to_string());
    let user = user.save(pool).await.expect("Failed to save user");

    let mut jar = Jar::test_jar(user.id, "selectjar");
    jar.title = "Select Test Jar".to_string();
    jar.description = Some("A jar for testing selections".to_string());
    let jar = jar.save(pool).await.expect("Failed to save jar");

    let mut donation = Donation::test_donation(jar.id, user.id, 42.0);
    donation.note = Some("Test donation".to_string());
    let donation = donation.save(pool).await.expect("Failed to save donation");

    (user, jar, donation)
}

#[tokio::test]
async fn test_user_select_id_and_email() {
    let pool = create_clean_db().await;
    let (user, _jar, _donation) = setup_select_test_data(&pool).await;

    let (id, email): (i64, String) = User::query()
        .filter(User::ID.eq(user.id))
        .select((User::ID, User::EMAIL))
        .fetch_one_as(&pool)
        .await
        .expect("Failed to select id and email");

    assert_eq!(id, user.id);
    assert_eq!(email, "select@example.com");
}

#[tokio::test]
async fn test_user_select_multiple_fields() {
    let pool = create_clean_db().await;
    let (user, _jar, _donation) = setup_select_test_data(&pool).await;

    let (id, email, username, first_name): (i64, String, String, String) = User::query()
        .filter(User::ID.eq(user.id))
        .select((User::ID, User::EMAIL, User::USERNAME, User::FIRST_NAME))
        .fetch_one_as(&pool)
        .await
        .expect("Failed to select multiple user fields");

    assert_eq!(id, user.id);
    assert_eq!(email, "select@example.com");
    assert_eq!(username, "selectuser");
    assert_eq!(first_name, "Test");
}

#[tokio::test]
async fn test_jar_select_with_foreign_key() {
    let pool = create_clean_db().await;
    let (user, jar, _donation) = setup_select_test_data(&pool).await;

    let (alias, owner_id, title): (String, i64, String) = Jar::query()
        .filter(Jar::ID.eq(jar.id))
        .select((Jar::ALIAS, Jar::OWNER_ID, Jar::TITLE))
        .fetch_one_as(&pool)
        .await
        .expect("Failed to select jar fields");

    assert_eq!(alias, "selectjar");
    assert_eq!(owner_id, user.id);
    assert_eq!(title, "Select Test Jar");
}

#[tokio::test]
async fn test_donation_select_with_uuid() {
    let pool = create_clean_db().await;
    let (_user, jar, donation) = setup_select_test_data(&pool).await;
    let (id, jar_id, amount): (Uuid, i64, f64) = Donation::query()
        .filter(Donation::ID.eq(donation.id.clone()))
        .select((Donation::ID, Donation::JAR_ID, Donation::AMOUNT))
        .fetch_one_as(&pool)
        .await
        .expect("Failed to select donation fields");

    assert_eq!(&id.to_string(), &donation.id.to_string());
    assert_eq!(jar_id, jar.id);
    assert_eq!(amount, 42.0);
}

#[tokio::test]
async fn test_select_nullable_fields() {
    let pool = create_clean_db().await;
    let (user, _jar, _donation) = setup_select_test_data(&pool).await;

    let (id, bio, wallpaper): (i64, Option<String>, Option<String>) = User::query()
        .filter(User::ID.eq(user.id))
        .select((User::ID, User::BIO, User::WALLPAPER_URL))
        .fetch_one_as(&pool)
        .await
        .expect("Failed to select nullable fields");

    assert_eq!(id, user.id);
    assert_eq!(bio, Some("A test bio".to_string()));
    assert!(wallpaper.is_none());
}

#[tokio::test]
async fn test_select_with_filtering() {
    let pool = create_clean_db().await;

    let _user1 = User::test_user("select1@example.com", "select1")
        .save(&pool)
        .await
        .expect("Failed to save user1");

    let _user2 = User::test_user("select2@example.com", "select2")
        .save(&pool)
        .await
        .expect("Failed to save user2");

    let (email, username): (String, String) = User::query()
        .filter(User::USERNAME.eq("select1".to_string()))
        .select((User::EMAIL, User::USERNAME))
        .fetch_one_as(&pool)
        .await
        .expect("Failed to select with filtering");

    assert_eq!(email, "select1@example.com");
    assert_eq!(username, "select1");

    let results: Vec<(String, String)> = User::query()
        .filter(User::EMAIL.like("%select%".to_string()))
        .select((User::EMAIL, User::USERNAME))
        .fetch_all_as(&pool)
        .await
        .expect("Failed to select multiple with filtering");

    assert!(results.len() >= 2);
    assert!(
        results
            .iter()
            .any(|(e, u)| e == "select1@example.com" && u == "select1")
    );
    assert!(
        results
            .iter()
            .any(|(e, u)| e == "select2@example.com" && u == "select2")
    );
}

#[tokio::test]
async fn test_select_boolean_and_numeric_fields() {
    let pool = create_clean_db().await;
    let (_user, jar, donation) = setup_select_test_data(&pool).await;

    let (minimal_donation, total_amount, total_donations): (f64, f64, i32) = Jar::query()
        .filter(Jar::ID.eq(jar.id))
        .select((
            Jar::MINIMAL_DONATION,
            Jar::TOTAL_AMOUNT,
            Jar::TOTAL_DONATIONS,
        ))
        .fetch_one_as(&pool)
        .await
        .expect("Failed to select numeric fields");

    assert_eq!(minimal_donation, 1.0);
    assert_eq!(total_amount, 0.0);
    assert_eq!(total_donations, 0);

    let (is_payed, is_refunded): (bool, bool) = Donation::query()
        .filter(Donation::ID.eq(donation.id))
        .select((Donation::IS_PAYED, Donation::IS_REFUNDED))
        .fetch_one_as(&pool)
        .await
        .expect("Failed to select boolean fields");

    assert_eq!(is_payed, true);
    assert_eq!(is_refunded, false);
}

#[tokio::test]
async fn test_select_timestamp_fields() {
    let pool = create_clean_db().await;
    let (user, _jar, _donation) = setup_select_test_data(&pool).await;

    let (created_at, updated_at): (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) =
        User::query()
            .filter(User::ID.eq(user.id))
            .select((User::CREATED_AT, User::UPDATED_AT))
            .fetch_one_as(&pool)
            .await
            .expect("Failed to select timestamp fields");

    let now = chrono::Utc::now();
    let one_minute_ago = now - chrono::Duration::minutes(1);

    assert!(created_at > one_minute_ago, "created_at should be recent");
    assert!(updated_at > one_minute_ago, "updated_at should be recent");
    assert!(created_at <= now, "created_at should not be in the future");
    assert!(updated_at <= now, "updated_at should not be in the future");
}
