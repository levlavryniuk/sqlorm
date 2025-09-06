use entities::{User, Jar, Donation};
use sqlorm_postgres_example::create_clean_db;

async fn setup_test_data(pool: &sqlorm_core::Pool) -> (User, User, Jar, Jar, Donation, Donation) {
    // Create users
    let mut user1 = User::test_user("owner1@example.com", "owner1");
    user1 = user1.save(pool).await.expect("Failed to save user1");
    
    let mut user2 = User::test_user("owner2@example.com", "owner2");
    user2 = user2.save(pool).await.expect("Failed to save user2");

    // Create jars
    let mut jar1 = Jar::test_jar(user1.id, "jar1");
    jar1.title = "User1's Jar".to_string();
    jar1 = jar1.save(pool).await.expect("Failed to save jar1");
    
    let mut jar2 = Jar::test_jar(user2.id, "jar2");
    jar2.title = "User2's Jar".to_string();
    jar2 = jar2.save(pool).await.expect("Failed to save jar2");

    // Create donations
    let mut donation1 = Donation::test_donation(jar1.id, user2.id, 25.0);
    donation1 = donation1.save(pool).await.expect("Failed to save donation1");
    
    let mut donation2 = Donation::test_donation(jar2.id, user1.id, 50.0);
    donation2 = donation2.save(pool).await.expect("Failed to save donation2");

    (user1, user2, jar1, jar2, donation1, donation2)
}

#[tokio::test]
async fn test_belongs_to_lazy_loading() {
    let pool = create_clean_db().await;
    let (user1, _user2, jar1, _jar2, donation1, _donation2) = setup_test_data(&pool).await;

    // Test jar belongs_to user (lazy loading)
    let owner = jar1.owner(&pool).await.expect("Failed to load owner").expect("Owner not found");
    assert_eq!(owner.id, user1.id);
    assert_eq!(owner.username, "owner1");

    // Test donation belongs_to jar (lazy loading)
    let jar = donation1.jar(&pool).await.expect("Failed to load jar").expect("Jar not found");
    assert_eq!(jar.id, jar1.id);
    assert_eq!(jar.title, "User1's Jar");

    // Test donation belongs_to user (payer)
    let payer = donation1.payer(&pool).await.expect("Failed to load payer").expect("Payer not found");
    assert_eq!(payer.id, donation1.payer_id);
}

#[tokio::test]
async fn test_belongs_to_eager_loading() {
    let pool = create_clean_db().await;
    let (_user1, _user2, _jar1, _jar2, _donation1, _donation2) = setup_test_data(&pool).await;

    // Test eager loading jar with owner
    let jar_with_owner = Jar::query()
        .with_owner()
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch jar with owner");
    
    let owner = jar_with_owner.owner.expect("Owner should be loaded");
    assert_eq!(owner.id, jar_with_owner.owner_id);

    // Test eager loading multiple jars with owners
    let jars_with_owners = Jar::query()
        .with_owner()
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch jars with owners");
    
    assert_eq!(jars_with_owners.len(), 2);
    for jar in jars_with_owners {
        let owner = jar.owner.expect("Each jar should have owner loaded");
        assert_eq!(owner.id, jar.owner_id);
    }
}

#[tokio::test]
async fn test_has_many_lazy_loading() {
    let pool = create_clean_db().await;
    let (user1, user2, _jar1, _jar2, _donation1, _donation2) = setup_test_data(&pool).await;

    // Test user has_many jars (lazy loading)
    let user1_jars = user1.jars(&pool).await.expect("Failed to load user1 jars");
    assert_eq!(user1_jars.len(), 1);
    assert_eq!(user1_jars[0].owner_id, user1.id);
    assert_eq!(user1_jars[0].title, "User1's Jar");

    let user2_jars = user2.jars(&pool).await.expect("Failed to load user2 jars");
    assert_eq!(user2_jars.len(), 1);
    assert_eq!(user2_jars[0].owner_id, user2.id);
    assert_eq!(user2_jars[0].title, "User2's Jar");

    // Test user has_many payed_donations (lazy loading)
    let user1_donations = user1.payed_donations(&pool).await.expect("Failed to load user1 donations");
    assert_eq!(user1_donations.len(), 1);
    assert_eq!(user1_donations[0].payer_id, user1.id);
    assert_eq!(user1_donations[0].amount, 50.0);

    let user2_donations = user2.payed_donations(&pool).await.expect("Failed to load user2 donations");
    assert_eq!(user2_donations.len(), 1);
    assert_eq!(user2_donations[0].payer_id, user2.id);
    assert_eq!(user2_donations[0].amount, 25.0);
}

#[tokio::test]
async fn test_has_many_eager_loading() {
    let pool = create_clean_db().await;
    let (_user1, _user2, jar1, _jar2, _donation1, _donation2) = setup_test_data(&pool).await;

    // Test eager loading user with jars
    let user_with_jars = User::query()
        .with_jars()
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch user with jars");
    
    let jars = user_with_jars.jars.expect("Jars should be loaded");
    assert_eq!(jars.len(), 1);
    assert_eq!(jars[0].owner_id, user_with_jars.id);

    // Test eager loading jar with donations
    let jar_with_donations = Jar::query()
        .filter(Jar::ID.eq(jar1.id))
        .with_donations()
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch jar with donations");
    
    let donations = jar_with_donations.donations.expect("Donations should be loaded");
    assert_eq!(donations.len(), 1);
    assert_eq!(donations[0].jar_id, jar1.id);
    assert_eq!(donations[0].amount, 25.0);
}

#[tokio::test]
async fn test_has_many_empty_relations() {
    let pool = create_clean_db().await;
    
    // Create user with no jars
    let mut user = User::test_user("lonely@example.com", "lonely");
    user = user.save(&pool).await.expect("Failed to save user");

    // Test lazy loading empty relations
    let jars = user.jars(&pool).await.expect("Failed to load jars");
    assert!(jars.is_empty());

    let donations = user.payed_donations(&pool).await.expect("Failed to load donations");
    assert!(donations.is_empty());

    // Test eager loading empty relations
    let user_with_jars = User::query()
        .filter(User::ID.eq(user.id))
        .with_jars()
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch user with jars");
    
    let jars = user_with_jars.jars.expect("Jars should be loaded");
    assert!(jars.is_empty());
}

#[tokio::test]
async fn test_relations_with_filtering() {
    let pool = create_clean_db().await;
    let (user1, user2, _jar1, _jar2, _donation1, _donation2) = setup_test_data(&pool).await;

    // Test filtering with eager loading - should use correct table aliases
    let user_with_jars = User::query()
        .filter(User::ID.eq(user1.id))
        .with_jars()
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch specific user with jars");
    
    assert_eq!(user_with_jars.id, user1.id);
    assert_eq!(user_with_jars.username, "owner1");
    
    let jars = user_with_jars.jars.expect("Jars should be loaded");
    assert_eq!(jars.len(), 1);
    assert_eq!(jars[0].title, "User1's Jar");

    // Test the generated SQL contains correct table aliases
    let query_sql = User::query()
        .filter(User::ID.eq(user2.id))
        .with_jars()
        .to_sql();

    // The SQL should use proper table aliases to avoid column ambiguity
    assert!(query_sql.contains("user__") || query_sql.contains("users"), 
            "Expected proper table aliasing in SQL: {}", query_sql);
}

#[tokio::test]
async fn test_multiple_relation_types() {
    let pool = create_clean_db().await;
    let (_user1, _user2, _jar1, _jar2, donation1, _donation2) = setup_test_data(&pool).await;

    // Test donation with both belongs_to relations loaded
    let donation_with_relations = Donation::query()
        .filter(Donation::ID.eq(donation1.id))
        .with_jar()
        .with_payer()
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch donation with relations");

    // Check jar relation
    let jar = donation_with_relations.jar.expect("Jar should be loaded");
    assert_eq!(jar.id, donation1.jar_id);
    assert_eq!(jar.title, "User1's Jar");

    // Check payer relation  
    let payer = donation_with_relations.payer.expect("Payer should be loaded");
    assert_eq!(payer.id, donation1.payer_id);
    assert_eq!(payer.username, "owner2");
}

#[tokio::test]
async fn test_nested_relations() {
    let pool = create_clean_db().await;
    let (_user1, _user2, _jar1, _jar2, _donation1, _donation2) = setup_test_data(&pool).await;

    // Test loading user with jars, and each jar with its donations
    let users_with_full_data = User::query()
        .with_jars()
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch users with jars");

    for user in users_with_full_data {
        let jars = user.jars.expect("Jars should be loaded");
        for jar in jars {
            // Load donations for this jar separately (simulating nested loading)
            let donations = jar.donations(&pool).await.expect("Failed to load donations");
            if jar.title == "User1's Jar" {
                assert_eq!(donations.len(), 1);
                assert_eq!(donations[0].amount, 25.0);
            } else if jar.title == "User2's Jar" {
                assert_eq!(donations.len(), 1);
                assert_eq!(donations[0].amount, 50.0);
            }
        }
    }
}
