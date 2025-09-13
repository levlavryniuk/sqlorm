use common::entities::{JarExecutor, UserExecutor};
mod common;

use common::create_clean_db;
use common::entities::{Jar, User};

async fn setup_test_users(pool: &sqlorm::Pool) -> Vec<User> {
    let mut users = vec![
        User::test_user("eq1@example.com", "eq1"),
        User::test_user("eq2@example.com", "eq2"),
        User::test_user("like_me@example.com", "like_me"),
        User::test_user("other@example.com", "other"),
    ];

    users[2].bio = Some("Has bio content".to_string());

    let mut saved_users = Vec::new();
    for user in users {
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

    let found = User::query()
        .filter(User::EMAIL.eq("eq1@example.com".to_string()))
        .fetch_one(&pool)
        .await
        .expect("Failed to filter by email");
    assert_eq!(found.id, u1.id);

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

    let gt_results = User::query()
        .filter(User::ID.gt(u1.id))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with gt");
    assert!(gt_results.iter().any(|u| u.id == u2.id));
    assert!(!gt_results.iter().any(|u| u.id == u1.id));

    let ge_results = User::query()
        .filter(User::ID.ge(u1.id))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with ge");
    assert!(ge_results.iter().any(|u| u.id == u1.id));
    assert!(ge_results.iter().any(|u| u.id == u2.id));

    let lt_results = User::query()
        .filter(User::ID.lt(u2.id))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with lt");
    assert!(lt_results.iter().any(|u| u.id == u1.id));
    assert!(!lt_results.iter().any(|u| u.id == u2.id));

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
    let u3 = &users[2];

    let results = User::query()
        .filter(User::ID.in_(vec![u1.id, u2.id]))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with in");
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|u| u.id == u1.id));
    assert!(results.iter().any(|u| u.id == u2.id));

    let results = User::query()
        .filter(User::ID.not_in(vec![u1.id, u2.id]))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with not_in");
    assert!(results.iter().any(|u| u.id == u3.id));
    assert!(!results.iter().any(|u| u.id == u1.id));
    assert!(!results.iter().any(|u| u.id == u2.id));

    std::panic::catch_unwind(|| {
        let _ = User::ID.in_(vec![]);
    })
    .expect_err("Empty IN list should panic");
}

#[tokio::test]
async fn test_filter_is_null_and_is_not_null() {
    let pool = create_clean_db().await;
    let _users = setup_test_users(&pool).await;

    let null_results = User::query()
        .filter(User::BIO.is_null())
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with is_null");

    assert!(null_results.len() >= 3);
    assert!(null_results.iter().any(|u| u.username == "eq1"));
    assert!(!null_results.iter().any(|u| u.username == "like_me"));

    let not_null_results = User::query()
        .filter(User::BIO.is_not_null())
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with is_not_null");

    assert!(not_null_results.iter().any(|u| u.username == "like_me"));
}

#[tokio::test]
async fn test_filter_between_and_not_between() {
    let pool = create_clean_db().await;
    let users = setup_test_users(&pool).await;
    let u1 = &users[0];
    let u2 = &users[1];
    let u3 = &users[2];

    let results = User::query()
        .filter(User::ID.between(u1.id, u3.id))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with between");
    assert!(results.iter().any(|u| u.id == u1.id));
    assert!(results.iter().any(|u| u.id == u2.id));
    assert!(results.iter().any(|u| u.id == u3.id));

    let results = User::query()
        .filter(User::ID.not_between(u1.id, u2.id))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with not_between");
    assert!(results.iter().any(|u| u.id == u3.id));
    assert!(!results.iter().any(|u| u.id == u1.id));
    assert!(!results.iter().any(|u| u.id == u2.id));

    let results = User::query()
        .filter(User::ID.between(u3.id, u1.id))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter with reversed between");
    assert!(
        results.is_empty(),
        "Between with reversed bounds should return no rows"
    );
}

#[tokio::test]
async fn test_multiple_filters() {
    let pool = create_clean_db().await;

    let user = User::test_user("owner@example.com", "owner")
        .save(&pool)
        .await
        .expect("Failed to save user");

    let _jar = Jar::test_jar(user.id, "jar_eq")
        .save(&pool)
        .await
        .expect("Failed to save jar");

    let results = Jar::query()
        .filter(Jar::ALIAS.eq("jar_eq".to_string()))
        .filter(Jar::OWNER_ID.eq(user.id))
        .fetch_all(&pool)
        .await
        .expect("Failed to filter jars");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].owner_id, user.id);
}
