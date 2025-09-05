// use chrono::Utc;
// use entities::{Donation, Jar, JarExecutor, JarRelations, User};
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
// #[tokio::test]
// async fn test_user_crud() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let user = User {
//         email: "test@example.com".into(),
//         password: "secret".into(),
//         username: "alice".into(),
//         first_name: "Alice".into(),
//         last_name: "Tester".into(),
//         ..Default::default()
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//
//     assert!(user.id > 0);
//
//     let found = User::find_by_email(&pool, "test@example.com".into())
//         .await
//         .unwrap()
//         .unwrap();
//     assert_eq!(found.username, "alice");
//
//     let updated = User {
//         username: "alice_updated".into(),
//         ..user
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//     assert_eq!(updated.username, "alice_updated");
//
//     let all = User::find_all(&pool).await.unwrap();
//     assert_eq!(all.len(), 1);
// }
//
// #[tokio::test]
// async fn test_jar_and_relation() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let user = User {
//         email: "jarowner@example.com".into(),
//         password: "secret".into(),
//         username: "jarowner".into(),
//         first_name: "Jar".into(),
//         last_name: "Owner".into(),
//         ..Default::default()
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//
//     let jar = Jar {
//         title: "My Jar".into(),
//         owner_id: user.id,
//         description: Some("Donation jar".into()),
//         minimal_donation: 2.0,
//         alias: "myjar".into(),
//         goal: Some(100.),
//         ..Default::default()
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//
//     let owner = jar.owner(&pool).await.unwrap().unwrap();
//     assert_eq!(owner.username, "jarowner");
//     let jar = Jar::query().with_owner().fetch_one(&pool).await.unwrap();
//     let owner = jar.owner.unwrap();
//     assert_eq!(jar.owner_id, owner.id);
// }
//
// #[tokio::test]
// async fn test_donation_relations() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let payer = User {
//         email: "payer@example.com".into(),
//         password: "secret".into(),
//         username: "payer".into(),
//         first_name: "Pay".into(),
//         last_name: "Er".into(),
//         ..Default::default()
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//
//     let owner = User {
//         email: "owner@example.com".into(),
//         password: "secret".into(),
//         username: "owner".into(),
//         first_name: "Jar".into(),
//         last_name: "Owner".into(),
//         ..Default::default()
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//
//     let jar = Jar {
//         owner_id: owner.id,
//         title: "Donation Jar".into(),
//         minimal_donation: 2.0,
//         alias: "donjar".into(),
//         ..Default::default()
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//
//     let donation = Donation {
//         amount: 10.0,
//         tip: 2.0,
//         jar_id: jar.id,
//         payer_id: payer.id,
//         is_payed: true,
//         transaction_id: Some("tx123".into()),
//         note: Some("Keep it up!".into()),
//         payed_at: Some(Utc::now()),
//         ..Default::default()
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//
//     let related_jar = donation.jar(&pool).await.unwrap().unwrap();
//     assert_eq!(related_jar.title, "Donation Jar");
//
//     let related_payer = donation.payer(&pool).await.unwrap().unwrap();
//     assert_eq!(related_payer.username, "payer");
// }
