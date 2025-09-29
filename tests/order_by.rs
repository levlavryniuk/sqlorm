mod common;

use common::create_clean_db;
use common::entities::User;
use sqlorm::GenericExecutor;

#[tokio::test]
async fn test_order_by_single_column_asc() {
    let pool = create_clean_db().await;

    let _user1 = User::test_user("order1@example.com", "zzz")
        .save(&pool)
        .await
        .expect("Failed to save user1");

    let _user2 = User::test_user("order2@example.com", "aaa")
        .save(&pool)
        .await
        .expect("Failed to save user2");

    let results: Vec<(String,)> = User::query()
        .select((User::USERNAME,))
        .order_by(User::USERNAME.asc())
        .fetch_all_as(&pool)
        .await
        .expect("Failed to select with order by asc");

    assert!(
        results.windows(2).all(|w| w[0] <= w[1]),
        "results are not sorted ascending: {:?}",
        results
    );
}

#[tokio::test]
async fn test_order_by_single_column_desc() {
    let pool = create_clean_db().await;

    let _user1 = User::test_user("order3@example.com", "aaa")
        .save(&pool)
        .await
        .expect("Failed to save user1");

    let _user2 = User::test_user("order4@example.com", "zzz")
        .save(&pool)
        .await
        .expect("Failed to save user2");

    let results: Vec<(String,)> = User::query()
        .select((User::USERNAME,))
        .order_by(User::USERNAME.desc())
        .fetch_all_as(&pool)
        .await
        .expect("Failed to select with order by desc");

    assert!(
        results.windows(2).all(|w| w[0] >= w[1]),
        "results are not sorted descending: {:?}",
        results
    );
}

#[tokio::test]
async fn test_order_by_multiple_columns() {
    let pool = create_clean_db().await;

    let mut user1 = User::test_user("multi1@example.com", "user_c");
    user1.first_name = "John".to_string();
    user1.save(&pool).await.expect("Failed to save user1");

    let mut user2 = User::test_user("multi2@example.com", "user_b");
    user2.first_name = "John".to_string();
    user2.save(&pool).await.expect("Failed to save user2");

    let mut user3 = User::test_user("multi3@example.com", "user_a");
    user3.first_name = "Alice".to_string();
    user3.save(&pool).await.expect("Failed to save user3");

    let results: Vec<(String, String)> = User::query()
        .select((User::FIRST_NAME, User::USERNAME))
        .order_by(User::FIRST_NAME.asc())
        .order_by(User::USERNAME.asc())
        .fetch_all_as(&pool)
        .await
        .expect("Failed to select with multiple order by");

    assert!(
        results.windows(2).all(|w| {
            if w[0].0 == w[1].0 {
                w[0].1 <= w[1].1
            } else {
                w[0].0 <= w[1].0
            }
        }),
        "results are not properly sorted by first_name ASC then username ASC: {:?}",
        results
    );
}
