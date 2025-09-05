// use entities::{User, UserExecutor};
// use shared::testing::create_test_db;
// use sqlx::{Pool, Postgres};
//
// async fn migrate(pool: &Pool<Postgres>) {
//     sqlx::migrate!("../../../migrations")
//         .run(pool)
//         .await
//         .expect("Failed to run migrations");
// }
//
// /// Helper to insert a user quickly
// async fn insert_user(
//     pool: &Pool<Postgres>,
//     email: &str,
//     username: &str,
//     bio: Option<&str>,
// ) -> User {
//     User {
//         email: email.into(),
//         password: "secret".into(),
//         username: username.into(),
//         first_name: "Test".into(),
//         last_name: "User".into(),
//         bio: bio.map(|s| s.to_string()),
//         ..Default::default()
//     }
//     .save(pool)
//     .await
//     .unwrap()
// }
//
// #[tokio::test]
// async fn test_filter_eq_and_ne() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let u1 = insert_user(&pool, "eq1@example.com", "eq1", None).await;
//     let u2 = insert_user(&pool, "eq2@example.com", "eq2", None).await;
//
//     // eq
//     let found = User::query()
//         .filter(User::EMAIL.eq("eq1@example.com".into()))
//         .fetch_one(&pool)
//         .await
//         .unwrap();
//     assert_eq!(found.id, u1.id);
//
//     // ne
//     let results = User::query()
//         .filter(User::EMAIL.ne("eq1@example.com".into()))
//         .fetch_all(&pool)
//         .await
//         .unwrap();
//     assert!(results.iter().any(|u| u.id == u2.id));
//     assert!(!results.iter().any(|u| u.id == u1.id));
// }
//
// #[tokio::test]
// async fn test_filter_gt_ge_lt_le() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let u1 = insert_user(&pool, "cmp1@example.com", "cmp1", None).await;
//     let u2 = insert_user(&pool, "cmp2@example.com", "cmp2", None).await;
//
//     // gt
//     let gt_results = User::query()
//         .filter(User::ID.gt(u1.id))
//         .fetch_all(&pool)
//         .await
//         .unwrap();
//     assert!(gt_results.iter().any(|u| u.id == u2.id));
//     assert!(!gt_results.iter().any(|u| u.id == u1.id));
//
//     // ge
//     let ge_results = User::query()
//         .filter(User::ID.ge(u1.id))
//         .fetch_all(&pool)
//         .await
//         .unwrap();
//     assert!(ge_results.iter().any(|u| u.id == u1.id));
//     assert!(ge_results.iter().any(|u| u.id == u2.id));
//
//     // lt
//     let lt_results = User::query()
//         .filter(User::ID.lt(u2.id))
//         .fetch_all(&pool)
//         .await
//         .unwrap();
//     assert!(lt_results.iter().any(|u| u.id == u1.id));
//     assert!(!lt_results.iter().any(|u| u.id == u2.id));
//
//     // le
//     let le_results = User::query()
//         .filter(User::ID.le(u2.id))
//         .fetch_all(&pool)
//         .await
//         .unwrap();
//     assert!(le_results.iter().any(|u| u.id == u1.id));
//     assert!(le_results.iter().any(|u| u.id == u2.id));
// }
//
// #[tokio::test]
// async fn test_filter_like() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     insert_user(&pool, "like_me@example.com", "like_me", None).await;
//     insert_user(&pool, "other@example.com", "other", None).await;
//
//     let results = User::query()
//         .filter(User::EMAIL.like("%like_me%".into()))
//         .fetch_all(&pool)
//         .await
//         .unwrap();
//
//     assert_eq!(results.len(), 1);
//     assert_eq!(results[0].username, "like_me");
// }
//
// #[tokio::test]
// async fn test_filter_in_and_not_in() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let u1 = insert_user(&pool, "in1@example.com", "in1", None).await;
//     let u2 = insert_user(&pool, "in2@example.com", "in2", None).await;
//     let _u3 = insert_user(&pool, "in3@example.com", "in3", None).await;
//
//     // in
//     let results = User::query()
//         .filter(User::ID.in_(vec![u1.id, u2.id]))
//         .fetch_all(&pool)
//         .await
//         .unwrap();
//     assert_eq!(results.len(), 2);
//
//     // not_in
//     let results = User::query()
//         .filter(User::ID.not_in(vec![u1.id, u2.id]))
//         .fetch_all(&pool)
//         .await
//         .unwrap();
//     assert_eq!(results.len(), 1);
//     assert_eq!(results[0].username, "in3");
//
//     // edge case: empty IN list → should fail or return error
//     let results = User::query()
//         .filter(User::ID.in_(vec![]))
//         .fetch_all(&pool)
//         .await;
//     assert!(results.is_err(), "Empty IN should fail or return error");
// }
//
// #[tokio::test]
// async fn test_filter_is_null_and_is_not_null() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     insert_user(&pool, "null1@example.com", "null1", None).await;
//     insert_user(&pool, "null2@example.com", "null2", Some("Has bio")).await;
//
//     let null_results = User::query()
//         .filter(User::BIO.is_null())
//         .fetch_all(&pool)
//         .await
//         .unwrap();
//     assert!(null_results.iter().any(|u| u.username == "null1"));
//     assert!(!null_results.iter().any(|u| u.username == "null2"));
//
//     let not_null_results = User::query()
//         .filter(User::BIO.is_not_null())
//         .fetch_all(&pool)
//         .await
//         .unwrap();
//     assert!(not_null_results.iter().any(|u| u.username == "null2"));
// }
//
// #[tokio::test]
// async fn test_filter_between_and_not_between() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let u1 = insert_user(&pool, "btw1@example.com", "btw1", None).await;
//     let u2 = insert_user(&pool, "btw2@example.com", "btw2", None).await;
//     let u3 = insert_user(&pool, "btw3@example.com", "btw3", None).await;
//
//     // between
//     let results = User::query()
//         .filter(User::ID.between(u1.id, u3.id))
//         .fetch_all(&pool)
//         .await
//         .unwrap();
//     assert!(results.iter().any(|u| u.id == u2.id));
//
//     // not_between
//     let results = User::query()
//         .filter(User::ID.not_between(u1.id, u2.id))
//         .fetch_all(&pool)
//         .await
//         .unwrap();
//     assert!(results.iter().any(|u| u.id == u3.id));
//
//     // edge case: reversed bounds
//     let results = User::query()
//         .filter(User::ID.between(u3.id, u1.id))
//         .fetch_all(&pool)
//         .await
//         .unwrap();
//     assert!(
//         results.is_empty(),
//         "Between with reversed bounds should return no rows"
//     );
// }
