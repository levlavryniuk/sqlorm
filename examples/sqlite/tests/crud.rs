use entities::{User, Jar, Donation};
use sqlorm_sqlite_example::create_clean_db;

#[tokio::test]
async fn test_user_crud_operations() {
    let pool = create_clean_db().await;

    // Test CREATE (insert new user)
    let mut user = User::test_user("test@example.com", "testuser");
    user = user.save(&pool).await.expect("Failed to save user");
    
    assert!(user.id > 0, "User ID should be auto-generated");
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.username, "testuser");

    // Test READ by primary key
    let found_user = User::find_by_id(&pool, user.id)
        .await
        .expect("Failed to find user by ID")
        .expect("User not found");
    assert_eq!(found_user.id, user.id);
    assert_eq!(found_user.email, "test@example.com");

    // Test READ by unique field (email)
    let found_by_email = User::find_by_email(&pool, "test@example.com".to_string())
        .await
        .expect("Failed to find user by email")
        .expect("User not found by email");
    assert_eq!(found_by_email.id, user.id);

    // Test UPDATE
    let mut updated_user = found_user;
    updated_user.username = "updated_username".to_string();
    updated_user = updated_user.save(&pool).await.expect("Failed to update user");
    assert_eq!(updated_user.username, "updated_username");

    // Verify the update persisted
    let verified_user = User::find_by_id(&pool, user.id)
        .await
        .expect("Failed to find updated user")
        .expect("Updated user not found");
    assert_eq!(verified_user.username, "updated_username");

    // Test READ ALL
    let all_users = User::find_all(&pool).await.expect("Failed to find all users");
    assert_eq!(all_users.len(), 1);
    assert_eq!(all_users[0].id, user.id);
}

#[tokio::test]
async fn test_insert_vs_update_behavior() {
    let pool = create_clean_db().await;

    // Test INSERT (when ID is default/0)
    let mut new_user = User {
        id: 0, // Default value triggers INSERT
        email: "insert@example.com".to_string(),
        username: "insert_user".to_string(),
        password: "secret".to_string(),
        first_name: "Insert".to_string(),
        last_name: "User".to_string(),
        ..Default::default()
    };
    
    let inserted = new_user.save(&pool).await.expect("Failed to insert user");
    assert!(inserted.id > 0, "Should have generated an ID");
    assert_eq!(inserted.email, "insert@example.com");

    // Test UPDATE (when ID is not default)
    let mut existing_user = inserted.clone();
    existing_user.username = "updated_insert_user".to_string();
    
    let updated = existing_user.save(&pool).await.expect("Failed to update user");
    assert_eq!(updated.id, inserted.id, "ID should remain the same");
    assert_eq!(updated.username, "updated_insert_user");

    // Verify only one user exists
    let all_users = User::find_all(&pool).await.expect("Failed to get all users");
    assert_eq!(all_users.len(), 1);
}

#[tokio::test]
async fn test_forced_insert_and_update() {
    let pool = create_clean_db().await;

    // Test forced INSERT
    let mut user = User::test_user("force@example.com", "forceuser");
    user = user.insert(&pool).await.expect("Failed to force insert");
    assert!(user.id > 0);

    // Test forced UPDATE
    let original_username = user.username.clone();
    user.username = "force_updated".to_string();
    user = user.update(&pool).await.expect("Failed to force update");
    assert_eq!(user.username, "force_updated");
    assert_ne!(user.username, original_username);
}

#[tokio::test]
async fn test_jar_with_foreign_key() {
    let pool = create_clean_db().await;

    // First create a user
    let mut user = User::test_user("jarowner@example.com", "jarowner");
    user = user.save(&pool).await.expect("Failed to save user");

    // Create a jar owned by this user
    let mut jar = Jar::test_jar(user.id, "testjar");
    jar = jar.save(&pool).await.expect("Failed to save jar");

    assert!(jar.id > 0);
    assert_eq!(jar.owner_id, user.id);
    assert_eq!(jar.alias, "testjar");

    // Test unique constraint on alias
    let found_jar = Jar::find_by_alias(&pool, "testjar".to_string())
        .await
        .expect("Failed to find jar by alias")
        .expect("Jar not found by alias");
    assert_eq!(found_jar.id, jar.id);
}

#[tokio::test]
async fn test_donation_with_uuid_primary_key() {
    let pool = create_clean_db().await;

    // Setup user and jar
    let mut user = User::test_user("donor@example.com", "donor");
    user = user.save(&pool).await.expect("Failed to save user");

    let mut jar = Jar::test_jar(user.id, "donationjar");
    jar = jar.save(&pool).await.expect("Failed to save jar");

    // Create donation
    let mut donation = Donation::test_donation(jar.id, user.id, 50.0);
    donation = donation.save(&pool).await.expect("Failed to save donation");

    // UUID should be generated and not be the default
    assert_ne!(donation.id.to_string(), "00000000-0000-0000-0000-000000000000");
    assert_eq!(donation.amount, 50.0);
    assert_eq!(donation.jar_id, jar.id);
    assert_eq!(donation.payer_id, user.id);

    // Test finding by UUID
    let found_donation = Donation::find_by_id(&pool, donation.id)
        .await
        .expect("Failed to find donation by UUID")
        .expect("Donation not found");
    assert_eq!(found_donation.id, donation.id);
    assert_eq!(found_donation.amount, 50.0);
}
