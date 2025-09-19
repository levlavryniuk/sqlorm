use common::entities::{JarExecutor, UserExecutor};
use sqlorm::GenericExecutor;
mod common;

use common::create_clean_db;
use common::entities::{Jar, User};

#[tokio::test]
async fn test_user_limit_results() {
    let pool = create_clean_db().await;

    for i in 0..5 {
        let email = format!("limit{i}@example.com");
        let username = format!("limit{i}");
        User::test_user(&email, &username)
            .save(&pool)
            .await
            .expect("Failed to save user");
    }

    let results: Vec<(String, String)> = User::query()
        .select((User::EMAIL, User::USERNAME))
        .limit(2)
        .fetch_all_as(&pool)
        .await
        .expect("Failed to select with limit");

    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_user_offset_results() {
    let pool = create_clean_db().await;

    for i in 0..5 {
        let email = format!("offset{i}@example.com");
        let username = format!("offset{i}");
        User::test_user(&email, &username)
            .save(&pool)
            .await
            .expect("Failed to save user");
    }

    let results: Vec<(String, String)> = User::query()
        .select((User::EMAIL, User::USERNAME))
        .offset(2)
        .fetch_all_as(&pool)
        .await
        .expect("Failed to select with offset");

    assert!(
        results
            .iter()
            .all(|(e, _)| !e.starts_with("offset0") && !e.starts_with("offset1"))
    );
}

#[tokio::test]
async fn test_user_limit_and_offset_results() {
    let pool = create_clean_db().await;

    for i in 0..10 {
        let email = format!("page{i}@example.com");
        let username = format!("page{i}");
        User::test_user(&email, &username)
            .save(&pool)
            .await
            .expect("Failed to save user");
    }

    let results: Vec<(String, String)> = User::query()
        .select((User::EMAIL, User::USERNAME))
        .offset(3)
        .limit(3)
        .fetch_all_as(&pool)
        .await
        .expect("Failed to select with limit+offset");

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].0, "page3@example.com");
    assert_eq!(results[1].0, "page4@example.com");
    assert_eq!(results[2].0, "page5@example.com");
}
