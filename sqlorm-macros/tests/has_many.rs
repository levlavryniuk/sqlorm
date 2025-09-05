// use chrono::Utc;
// use entities::User;
// use entities::{Donation, Jar};
// use macros_core::Executor;
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
// async fn test_user_has_many_jars() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let mut user = User {
//         id: 0,
//         email: "owner@example.com".into(),
//         password: "secret".into(),
//         username: "owner".into(),
//         first_name: "Jar".into(),
//         last_name: "Owner".into(),
//         ..Default::default()
//     };
//     let user = user.save(&pool).await.unwrap();
//
//     let jar1 = Jar {
//         id: 0,
//         title: "Jar One".into(),
//         description: None,
//         minimal_donation: 1.0,
//         total_amount: 0.0,
//         total_donations: 0,
//         alias: "jar1".into(),
//         goal: None,
//         owner_id: user.id,
//         ..Default::default()
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//
//     let jar2 = Jar {
//         id: 0,
//         title: "Jar Two".into(),
//         description: None,
//         minimal_donation: 2.0,
//         total_amount: 0.0,
//         total_donations: 0,
//         alias: "jar2".into(),
//         goal: None,
//         owner_id: user.id,
//         ..Default::default()
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//
//     let jars = user.jars(&pool).await.unwrap();
//     assert_eq!(jars.len(), 2);
//     assert!(jars.iter().any(|j| j.id == jar1.id));
//     assert!(jars.iter().any(|j| j.id == jar2.id));
// }
//
// #[tokio::test]
// async fn test_user_has_many_payed_donations() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let payer = User {
//         id: 0,
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
//         id: 0,
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
//         id: 0,
//         title: "Donation Jar".into(),
//         description: None,
//         minimal_donation: 1.0,
//         total_amount: 0.0,
//         total_donations: 0,
//         alias: "donjar".into(),
//         goal: None,
//         owner_id: owner.id,
//         ..Default::default()
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//
//     let donation1 = Donation {
//         amount: 5.0,
//         tip: 1.0,
//         jar_id: jar.id,
//         payer_id: payer.id,
//         is_payed: true,
//         transaction_id: Some("tx1".into()),
//         note: None,
//         is_refunded: false,
//         refunded_at: None,
//         deleted_at: None,
//         payed_at: Some(Utc::now()),
//         ..Default::default()
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//
//     let donation2 = Donation {
//         amount: 10.0,
//         tip: 2.0,
//         jar_id: jar.id,
//         payer_id: payer.id,
//         is_payed: true,
//         transaction_id: Some("tx2".into()),
//         note: Some("Great work!".into()),
//         is_refunded: false,
//         refunded_at: None,
//         deleted_at: None,
//         created_at: Utc::now(),
//         updated_at: Utc::now(),
//         payed_at: Some(Utc::now()),
//         ..Default::default()
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//
//     let donations = payer.payed_donations(&pool).await.unwrap();
//     assert_eq!(donations.len(), 2);
//     assert!(donations.iter().any(|d| d.id == donation1.id));
//     assert!(donations.iter().any(|d| d.id == donation2.id));
// }
//
// #[tokio::test]
// async fn test_jar_has_many_donations() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let payer = User {
//         id: 0,
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
//         id: 0,
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
//         id: 0,
//         title: "Donation Jar".into(),
//         description: None,
//         minimal_donation: 1.0,
//         total_amount: 0.0,
//         total_donations: 0,
//         alias: "donjar".into(),
//         goal: None,
//         owner_id: owner.id,
//         ..Default::default()
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//
//     let donation1 = Donation {
//         amount: 5.0,
//         tip: 1.0,
//         jar_id: jar.id,
//         payer_id: payer.id,
//         is_payed: true,
//         transaction_id: Some("tx1".into()),
//         note: None,
//         is_refunded: false,
//         refunded_at: None,
//         deleted_at: None,
//         created_at: Utc::now(),
//         updated_at: Utc::now(),
//         payed_at: Some(Utc::now()),
//         ..Default::default()
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//
//     let donation2 = Donation {
//         amount: 15.0,
//         tip: 3.0,
//         jar_id: jar.id,
//         payer_id: payer.id,
//         is_payed: true,
//         transaction_id: Some("tx2".into()),
//         note: Some("Keep going!".into()),
//         is_refunded: false,
//         refunded_at: None,
//         deleted_at: None,
//         created_at: Utc::now(),
//         updated_at: Utc::now(),
//         payed_at: Some(Utc::now()),
//         ..Default::default()
//     }
//     .save(&pool)
//     .await
//     .unwrap();
//     let (id, email): (i64, String) = User::query()
//         .filter(User::ID.eq(owner.id))
//         .select(vec![User::ID.as_ref(), User::EMAIL.as_ref()])
//         .fetch_one_as(&pool)
//         .await
//         .unwrap();
//
//     assert_eq!(id, owner.id);
//     assert_eq!(&email, &owner.email);
//
//     let donations = jar.donations(&pool).await.unwrap();
//     assert_eq!(donations.len(), 2);
//     assert!(donations.iter().any(|d| d.id == donation1.id));
//     assert!(donations.iter().any(|d| d.id == donation2.id));
// }
