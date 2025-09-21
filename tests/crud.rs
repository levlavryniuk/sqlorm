mod common;
use common::entities::UserExecutor;
use sqlorm::StatementExecutor;

use common::create_clean_db;
use common::entities::{Donation, Jar, User};

#[tokio::test]
async fn test_user_crud_operations() {
    let pool = create_clean_db().await;

    let user = User::test_user("test@example.com", "testuser")
        .save(&pool)
        .await
        .expect("Failed to save user");

    assert!(user.id > 0, "User ID should be auto-generated");
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.username, "testuser");

    let mut found_user = User::find_by_id(&pool, user.id)
        .await
        .expect("Failed to find user by ID")
        .expect("User not found");
    assert_eq!(found_user.id, user.id);
    assert_eq!(found_user.email, "test@example.com");

    let found_by_email = User::find_by_email(&pool, "test@example.com".to_string())
        .await
        .expect("Failed to find user by email")
        .expect("User not found by email");
    assert_eq!(found_by_email.id, user.id);

    found_user.username = "updated_username".to_string();
    let updated_user = found_user
        .update()
        .columns(User::USERNAME)
        .execute(&pool)
        .await
        .expect("Failed to update user");
    assert_eq!(updated_user.username, "updated_username");

    let verified_user = User::find_by_id(&pool, user.id)
        .await
        .expect("Failed to find updated user")
        .expect("Updated user not found");
    assert_eq!(verified_user.username, "updated_username");

    let all_users = User::query()
        .fetch_all(&pool)
        .await
        .expect("Failed to find all users");
    assert_eq!(all_users.len(), 1);
    assert_eq!(all_users[0].id, user.id);
}

#[tokio::test]
async fn test_insert_vs_update_behavior() {
    let pool = create_clean_db().await;

    let new_user = User {
        email: "insert@example.com".to_string(),
        username: "insert_user".to_string(),
        password: "secret".to_string(),
        first_name: "Insert".to_string(),
        last_name: "User".to_string(),
        ..Default::default()
    }
    .save(&pool)
    .await
    .expect("Failed to insert user");
    assert!(new_user.id > 0, "Should have generated an ID");
    assert_eq!(new_user.email, "insert@example.com");

    let mut existing_user = new_user.clone();
    existing_user.username = "updated_insert_user".to_string();

    let updated = existing_user
        .save(&pool)
        .await
        .expect("Failed to update user");
    assert_eq!(updated.id, new_user.id, "ID should remain the same");
    assert_eq!(updated.username, "updated_insert_user");

    let all_users = User::query()
        .fetch_all(&pool)
        .await
        .expect("Failed to get all users");
    assert_eq!(all_users.len(), 1);
}

#[tokio::test]
async fn test_forced_insert_and_update() {
    let pool = create_clean_db().await;

    let user = User::test_user("force@example.com", "forceuser")
        .insert(&pool)
        .await
        .expect("Failed to force insert");
    let original_username = user.username.clone();
    let mut user_to_update = user;
    user_to_update.username = "force_updated".to_string();
    let user = user_to_update
        .update()
        .execute(&pool)
        .await
        .expect("Failed to force update");
    assert_eq!(user.username, "force_updated");
    assert_ne!(user.username, original_username);
}

#[tokio::test]
async fn test_jar_with_foreign_key() {
    let pool = create_clean_db().await;

    let user = User::test_user("jarowner@example.com", "jarowner")
        .save(&pool)
        .await
        .expect("Failed to save user");

    let jar = Jar::test_jar(user.id, "testjar")
        .save(&pool)
        .await
        .expect("Failed to save jar");

    assert!(jar.id > 0);
    assert_eq!(&jar.owner_id, &user.id);
    assert_eq!(&jar.alias, "testjar");

    let found_jar = Jar::find_by_alias(&pool, "testjar".to_string())
        .await
        .expect("Failed to find jar by alias")
        .expect("Jar not found by alias");
    assert_eq!(found_jar.id, jar.id);
}

#[tokio::test]
async fn test_donation_with_uuid_primary_key() {
    let pool = create_clean_db().await;

    let user = User::test_user("donor@example.com", "donor")
        .save(&pool)
        .await
        .expect("Failed to save user");

    let jar = Jar::test_jar(user.id, "donationjar")
        .save(&pool)
        .await
        .expect("Failed to save jar");

    let donation = Donation::test_donation(jar.id, user.id, 50.0)
        .save(&pool)
        .await
        .expect("Failed to save donation");

    assert_ne!(
        donation.id.to_string(),
        "00000000-0000-0000-0000-000000000000"
    );
    assert_eq!(donation.amount, 50.0);
    assert_eq!(donation.jar_id, jar.id);
    assert_eq!(donation.payer_id, user.id);

    let found_donation = Donation::find_by_id(&pool, donation.id.clone())
        .await
        .expect("Failed to find donation by UUID")
        .expect("Donation not found");
    assert_eq!(&found_donation.id, &donation.id);
    assert_eq!(found_donation.amount, 50.0);
}

#[tokio::test]
async fn test_delete_user() {
    let pool = create_clean_db().await;

    let user = User::test_user("test@example.com", "testuser")
        .save(&pool)
        .await
        .expect("Failed to save user");

    assert!(user.id > 0, "User ID should be auto-generated");
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.username, "testuser");

    let users = User::query().fetch_all(&pool).await.unwrap();
    assert_eq!(users.len(), 1);

    // soft delete, since User::deleted_at exists
    user.delete(&pool).await.unwrap();

    let user = User::query().fetch_one(&pool).await.unwrap();
    assert!(user.deleted_at.is_some());
}
