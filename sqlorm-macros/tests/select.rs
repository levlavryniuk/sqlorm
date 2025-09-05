// use entities::{Donation, Jar, User};
// use macros_core::Executor;
// use shared::testing::create_test_db;
// use sqlx::{Pool, Postgres};
// use uuid::Uuid;
//
// async fn migrate(pool: &Pool<Postgres>) {
//     sqlx::migrate!("../../../migrations")
//         .run(pool)
//         .await
//         .expect("Failed to run migrations");
// }
//
// async fn insert_user(pool: &Pool<Postgres>, email: &str, username: &str) -> User {
//     User {
//         email: email.into(),
//         password: "secret".into(),
//         username: username.into(),
//         first_name: "Test".into(),
//         last_name: "User".into(),
//         ..Default::default()
//     }
//     .save(pool)
//     .await
//     .unwrap()
// }
//
// async fn insert_jar(pool: &Pool<Postgres>, owner_id: i64, alias: &str) -> Jar {
//     Jar {
//         title: "Test Jar".into(),
//         description: Some("A test jar".into()),
//         minimal_donation: 1.0,
//         total_amount: 0.0,
//         total_donations: 0,
//         alias: alias.into(),
//         owner_id,
//         ..Default::default()
//     }
//     .save(pool)
//     .await
//     .unwrap()
// }
//
// async fn insert_donation(pool: &Pool<Postgres>, jar_id: i64, payer_id: i64) -> Donation {
//     Donation {
//         amount: 10.0,
//         tip: 1.0,
//         jar_id,
//         payer_id,
//         is_payed: true,
//         is_refunded: false,
//         ..Default::default()
//     }
//     .save(pool)
//     .await
//     .unwrap()
// }
//
// #[tokio::test]
// async fn test_user_select_id_and_email() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let u = insert_user(&pool, "col1@example.com", "col1").await;
//
//     let (id, email): (i64, String) = User::query()
//         .filter(User::ID.eq(u.id))
//         .select(vec![User::ID.as_ref(), User::EMAIL.as_ref()])
//         .fetch_one_as(&pool)
//         .await
//         .unwrap();
//
//     assert_eq!(id, u.id);
//     assert_eq!(email, "col1@example.com");
// }
//
// #[tokio::test]
// async fn test_jar_select_owner_relation_column() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let u = insert_user(&pool, "col2@example.com", "col2").await;
//     let j = insert_jar(&pool, u.id, "jar_alias").await;
//
//     let (alias, owner_id): (String, i64) = Jar::query()
//         .filter(Jar::ID.eq(j.id))
//         .select(vec![Jar::ALIAS.as_ref(), Jar::OWNER_ID.as_ref()])
//         .fetch_one_as(&pool)
//         .await
//         .unwrap();
//
//     assert_eq!(alias, "jar_alias");
//     assert_eq!(owner_id, u.id);
// }
//
// #[tokio::test]
// async fn test_donation_select_foreign_keys() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let u = insert_user(&pool, "col3@example.com", "col3").await;
//     let j = insert_jar(&pool, u.id, "donation_jar").await;
//     let d = insert_donation(&pool, j.id, u.id).await;
//
//     let (id, jar_id, payer_id): (Uuid, i64, i64) = Donation::query()
//         .filter(Donation::ID.eq(d.id))
//         .select(vec![
//             Donation::ID.as_ref(),
//             Donation::JAR_ID.as_ref(),
//             Donation::PAYER_ID.as_ref(),
//         ])
//         .fetch_one_as(&pool)
//         .await
//         .unwrap();
//
//     assert_eq!(id, d.id);
//     assert_eq!(jar_id, j.id);
//     assert_eq!(payer_id, u.id);
// }
//
// #[tokio::test]
// async fn test_select_nullable_columns() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let u = insert_user(&pool, "col4@example.com", "col4").await;
//
//     let (id, wallpaper): (i64, Option<String>) = User::query()
//         .filter(User::ID.eq(u.id))
//         .select(vec![User::ID.as_ref(), User::WALLPAPER_URL.as_ref()])
//         .fetch_one_as(&pool)
//         .await
//         .unwrap();
//
//     assert_eq!(id, u.id);
//     assert!(wallpaper.is_none());
// }
//
// #[tokio::test]
// async fn test_select_no_columns_should_fail() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     insert_user(&pool, "col5@example.com", "col5").await;
//
//     let result: Result<(), _> = User::query()
//         .select(vec![]) // no columns
//         .fetch_one_as(&pool)
//         .await;
//
//     assert!(result.is_ok(), "Selecting no columns should return ()");
// }
