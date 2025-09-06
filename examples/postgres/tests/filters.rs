use entities::{User, Jar};
use sqlorm_postgres_example::create_clean_db;

async fn setup_test_users(pool: &sqlorm_core::Pool) -> Vec<User> {
    let mut users = vec![
        User::test_user("eq1@example.com", "eq1"),
        User::test_user("eq2@example.com", "eq2"),
        User::test_user("like_me@example.com", "like_me"),
        User::test_user("other@example.com", "other"),
    ];

    // Set bio for some users
    users[2].bio = Some("Has bio content".to_string());
    
    let mut saved_users = Vec::new();
    for mut user in users {
        saved_users.push(user.save(pool).await.expect("Failed to save user"));
    }
    saved_users
}

#[tokio::test]
async fn test_filter_eq_and_ne() {
    let pool = create_clean_db().await;
    let users = setup_test_users(&pool).await;
    let u1 = &users[0];
    let u2 = &users[1];

    // Test EQ filter
    let found = User::query()
        .filter(User::EMAIL.eq("eq1@example.com".to_string()))
        .fetch_one(&pool)
        .await
        .expect("Failed to filter by email");
    assert_eq!(found.id, u1.id);

    // Test NE filter
    let results = User::query()
        .filter(User::EMAIL.ne("eq1@example.com".to_string()))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with ne");
    assert!(results.iter().any(|u| u.id == u2.id));
    assert!(!results.iter().any(|u| u.id == u1.id));
}

#[tokio::test]
async fn test_filter_comparison_operators() {
    let pool = create_clean_db().await;
    let users = setup_test_users(&pool).await;
    let u1 = &users[0];
    let u2 = &users[1];

    // Test GT (greater than)
    let gt_results = User::query()
        .filter(User::ID.gt(u1.id))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with gt");
    assert!(gt_results.iter().any(|u| u.id == u2.id));
    assert!(!gt_results.iter().any(|u| u.id == u1.id));

    // Test GE (greater than or equal)
    let ge_results = User::query()
        .filter(User::ID.ge(u1.id))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with ge");
    assert!(ge_results.iter().any(|u| u.id == u1.id));
    assert!(ge_results.iter().any(|u| u.id == u2.id));

    // Test LT (less than)
    let lt_results = User::query()
        .filter(User::ID.lt(u2.id))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with lt");
    assert!(lt_results.iter().any(|u| u.id == u1.id));
    assert!(!lt_results.iter().any(|u| u.id == u2.id));

    // Test LE (less than or equal)
    let le_results = User::query()
        .filter(User::ID.le(u2.id))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with le");
    assert!(le_results.iter().any(|u| u.id == u1.id));
    assert!(le_results.iter().any(|u| u.id == u2.id));
}

#[tokio::test]
async fn test_filter_like() {
    let pool = create_clean_db().await;
    let _users = setup_test_users(&pool).await;

    let results = User::query()
        .filter(User::EMAIL.like("%like_me%".to_string()))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with like");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].username, "like_me");
}

#[tokio::test]
async fn test_filter_in_and_not_in() {
    let pool = create_clean_db().await;
    let users = setup_test_users(&pool).await;
    let u1 = &users[0];
    let u2 = &users[1];
    let u3 = &users[2]; // "like_me" user

    // Test IN filter
    let results = User::query()
        .filter(User::ID.in_(vec![u1.id, u2.id]))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with in");
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|u| u.id == u1.id));
    assert!(results.iter().any(|u| u.id == u2.id));

    // Test NOT IN filter
    let results = User::query()
        .filter(User::ID.not_in(vec![u1.id, u2.id]))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with not_in");
    assert!(results.iter().any(|u| u.id == u3.id));
    assert!(!results.iter().any(|u| u.id == u1.id));
    assert!(!results.iter().any(|u| u.id == u2.id));

    // Test empty IN list (should fail)
    let result = User::query()
        .filter(User::ID.in_(vec![]))
        .fetch_all(&pool)
        .await;
    assert!(result.is_err(), "Empty IN should fail or return error");
}

#[tokio::test]
async fn test_filter_is_null_and_is_not_null() {
    let pool = create_clean_db().await;
    let _users = setup_test_users(&pool).await;

    // Find users with null bio
    let null_results = User::query()
        .filter(User::BIO.is_null())
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with is_null");
    
    // Should find users without bio (eq1, eq2, other)
    assert!(null_results.len() >= 3);
    assert!(null_results.iter().any(|u| u.username == "eq1"));
    assert!(!null_results.iter().any(|u| u.username == "like_me"));

    // Find users with non-null bio
    let not_null_results = User::query()
        .filter(User::BIO.is_not_null())
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with is_not_null");
        
    // Should find the "like_me" user who has a bio
    assert!(not_null_results.iter().any(|u| u.username == "like_me"));
}

#[tokio::test]
async fn test_filter_between_and_not_between() {
    let pool = create_clean_db().await;
    let users = setup_test_users(&pool).await;
    let u1 = &users[0];
    let u2 = &users[1];
    let u3 = &users[2];

    // Test BETWEEN
    let results = User::query()
        .filter(User::ID.between(u1.id, u3.id))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with between");
    assert!(results.iter().any(|u| u.id == u1.id));
    assert!(results.iter().any(|u| u.id == u2.id));
    assert!(results.iter().any(|u| u.id == u3.id));

    // Test NOT BETWEEN
    let results = User::query()
        .filter(User::ID.not_between(u1.id, u2.id))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with not_between");
    assert!(results.iter().any(|u| u.id == u3.id));
    // Should not contain u1 and u2
    assert!(!results.iter().any(|u| u.id == u1.id));
    assert!(!results.iter().any(|u| u.id == u2.id));

    // Test reversed bounds (should return no rows)
    let results = User::query()
        .filter(User::ID.between(u3.id, u1.id))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with reversed between");
    assert!(results.is_empty(), "Between with reversed bounds should return no rows");
}

#[tokio::test]
async fn test_multiple_filters() {
    let pool = create_clean_db().await;
    
    // Create a user and jar
    let mut user = User::test_user("owner@example.com", "owner");
    user = user.save(&pool).await.expect("Failed to save user");
    
    let mut jar1 = Jar::test_jar(user.id, "jar1");
    jar1.title = "Expensive Jar".to_string();
    jar1.minimal_donation = 10.0;
    jar1 = jar1.save(&pool).await.expect("Failed to save jar1");
    
    let mut jar2 = Jar::test_jar(user.id, "jar2");
    jar2.title = "Cheap Jar".to_string();
    jar2.minimal_donation = 1.0;
    jar2 = jar2.save(&pool).await.expect("Failed to save jar2");

    // Test combining multiple filters
    let results = Jar::query()
        .filter(Jar::OWNER_ID.eq(user.id))
        .filter(Jar::MINIMAL_DONATION.ge(5.0))
        .fetch_all(&pool)
        .await
        .expect("Failed to apply multiple filters");
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, jar1.id);
    assert_eq!(results[0].title, "Expensive Jar");
}
