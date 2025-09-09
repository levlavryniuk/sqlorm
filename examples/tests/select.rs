use entities::{Donation, Jar, User};
use sqlorm::Executor;
use sqlorm_examples::create_clean_db;
use uuid::Uuid;

#[cfg(feature = "postgres")]
type NotReallyUuid = uuid::Uuid;
#[cfg(feature = "sqlite")]
type NotReallyUuid = String;

async fn setup_select_test_data(pool: &sqlorm::Pool) -> (User, Jar, Donation) {
let mut user = User::test_user("select@example.com", "selectuser");
    user.bio = Some("A test bio".to_string());
    let user = user.save(pool).await.expect("Failed to save user");

    let mut jar = Jar::test_jar(user.id, "selectjar");
    jar.title = "Select Test Jar".to_string();
    jar.description = Some("A jar for testing selections".to_string());
    jar = jar.save(pool).await.expect("Failed to save jar");

    let mut donation = Donation::test_donation(jar.id, user.id, 42.0);
    donation.note = Some("Test donation".to_string());
    donation = donation.save(pool).await.expect("Failed to save donation");

    (user, jar, donation)
}

#[tokio::test]
async fn test_user_select_id_and_email() {
    let pool = create_clean_db().await;
    let (user, _jar, _donation) = setup_select_test_data(&pool).await;

    let (id, email): (i64, String) = User::query()
        .filter(User::ID.eq(user.id))
        .select(vec![User::ID.as_ref(), User::EMAIL.as_ref()])
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
        .select(vec![
            User::ID.as_ref(),
            User::EMAIL.as_ref(),
            User::USERNAME.as_ref(),
            User::FIRST_NAME.as_ref(),
        ])
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
        .select(vec![
            Jar::ALIAS.as_ref(),
            Jar::OWNER_ID.as_ref(),
            Jar::TITLE.as_ref(),
        ])
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
        .select(vec![
            Donation::ID.as_ref(),
            Donation::JAR_ID.as_ref(),
            Donation::AMOUNT.as_ref(),
        ])
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
        .select(vec![
            User::ID.as_ref(),
            User::BIO.as_ref(),
            User::WALLPAPER_URL.as_ref(),
        ])
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

    // Create multiple users
    let user1 = User::test_user("select1@example.com", "select1")
        .save(&pool)
        .await
        .expect("Failed to save user1");

    let user2 = User::test_user("select2@example.com", "select2")
        .save(&pool)
        .await
        .expect("Failed to save user2");

    // Select specific user with filtering
    let (email, username): (String, String) = User::query()
        .filter(User::USERNAME.eq("select1".to_string()))
        .select(vec![User::EMAIL.as_ref(), User::USERNAME.as_ref()])
        .fetch_one_as(&pool)
        .await
        .expect("Failed to select with filtering");

    assert_eq!(email, "select1@example.com");
    assert_eq!(username, "select1");

    // Select multiple users with filtering
    let results: Vec<(String, String)> = User::query()
        .filter(User::EMAIL.like("%select%".to_string()))
        .select(vec![User::EMAIL.as_ref(), User::USERNAME.as_ref()])
        .fetch_all_as(&pool)
        .await
        .expect("Failed to select multiple with filtering");

    assert!(results.len() >= 2);
    assert!(results
        .iter()
        .any(|(e, u)| e == "select1@example.com" && u == "select1"));
    assert!(results
        .iter()
        .any(|(e, u)| e == "select2@example.com" && u == "select2"));
}

#[tokio::test]
async fn test_select_boolean_and_numeric_fields() {
    let pool = create_clean_db().await;
    let (_user, jar, donation) = setup_select_test_data(&pool).await;

    let (minimal_donation, total_amount, total_donations): (f64, f64, i32) = Jar::query()
        .filter(Jar::ID.eq(jar.id))
        .select(vec![
            Jar::MINIMAL_DONATION.as_ref(),
            Jar::TOTAL_AMOUNT.as_ref(),
            Jar::TOTAL_DONATIONS.as_ref(),
        ])
        .fetch_one_as(&pool)
        .await
        .expect("Failed to select numeric fields");

    assert_eq!(minimal_donation, 1.0);
    assert_eq!(total_amount, 0.0);
    assert_eq!(total_donations, 0);

    let (is_payed, is_refunded): (bool, bool) = Donation::query()
        .filter(Donation::ID.eq(donation.id))
        .select(vec![
            Donation::IS_PAYED.as_ref(),
            Donation::IS_REFUNDED.as_ref(),
        ])
        .fetch_one_as(&pool)
        .await
        .expect("Failed to select boolean fields");

    assert_eq!(is_payed, true);
    assert_eq!(is_refunded, false);
}

#[tokio::test]
async fn test_select_timestamp_fields() {
    let pool = create_clean_db().await;
    let (user, _jar, donation) = setup_select_test_data(&pool).await;

    let (created_at, updated_at): (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) =
        User::query()
            .filter(User::ID.eq(user.id))
            .select(vec![User::CREATED_AT.as_ref(), User::UPDATED_AT.as_ref()])
            .fetch_one_as(&pool)
            .await
            .expect("Failed to select timestamp fields");

    // Timestamps should be recent (within last minute)
    let now = chrono::Utc::now();
    assert!(now.signed_duration_since(created_at).num_seconds() < 60);
    assert!(now.signed_duration_since(updated_at).num_seconds() < 60);

    // Test optional timestamp fields from donation
    let (payed_at, refunded_at): (
        Option<chrono::DateTime<chrono::Utc>>,
        Option<chrono::DateTime<chrono::Utc>>,
    ) = Donation::query()
        .filter(Donation::ID.eq(donation.id))
        .select(vec![
            Donation::PAYED_AT.as_ref(),
            Donation::REFUNDED_AT.as_ref(),
        ])
        .fetch_one_as(&pool)
        .await
        .expect("Failed to select optional timestamp fields");

    // payed_at might be set, refunded_at should be null
    assert!(refunded_at.is_none());
}

#[tokio::test]
async fn test_select_no_columns() {
    let _pool = create_clean_db().await;
    let _setup = setup_select_test_data(&_pool).await;

    // This should panic now that we've added validation for empty select
    let result = std::panic::catch_unwind(|| {
        let query = User::query();
        query.select::<()>(vec![])
    });
    assert!(result.is_err(), "Empty select should panic");
}

#[tokio::test]
async fn test_select_single_column() {
    let pool = create_clean_db().await;
    let (user, _jar, _donation) = setup_select_test_data(&pool).await;

    // Test selecting a single column
    let email: (String,) = User::query()
        .filter(User::ID.eq(user.id))
        .select(vec![User::EMAIL.as_ref()])
        .fetch_one_as(&pool)
        .await
        .expect("Failed to select single column");

    assert_eq!(email.0, "select@example.com");

    // Test selecting a single numeric column
    let id: (i64,) = User::query()
        .filter(User::EMAIL.eq("select@example.com".to_string()))
        .select(vec![User::ID.as_ref()])
        .fetch_one_as(&pool)
        .await
        .expect("Failed to select single numeric column");

    assert_eq!(id.0, user.id);
}
