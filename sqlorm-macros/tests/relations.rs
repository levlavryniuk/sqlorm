// use entities::{Jar, JarExecutor, JarRelations, User, UserExecutor, UserRelations};
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
// pub async fn insert_user(pool: &Pool<Postgres>, email: &str, username: &str) -> User {
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
// pub async fn insert_jar(pool: &Pool<Postgres>, owner_id: i64, title: &str) -> Jar {
//     Jar {
//         title: title.into(),
//         owner_id,
//         minimal_donation: 1.0,
//         alias: format!("alias_{}", title),
//         ..Default::default()
//     }
//     .save(pool)
//     .await
//     .unwrap()
// }
//
// #[tokio::test]
// async fn test_belongs_to_eager_fetch_one() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let user = insert_user(&pool, "owner@example.com", "owner").await;
//     insert_jar(&pool, user.id, "Jar One").await;
//
//     let jar_with_owner = Jar::query().with_owner().fetch_one(&pool).await.unwrap();
//     assert_eq!(jar_with_owner.owner.unwrap().id, user.id);
// }
//
// #[tokio::test]
// async fn test_belongs_to_eager_fetch_all() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let user1 = insert_user(&pool, "u1@example.com", "u1").await;
//     let user2 = insert_user(&pool, "u2@example.com", "u2").await;
//
//     insert_jar(&pool, user1.id, "Jar1").await;
//     insert_jar(&pool, user2.id, "Jar2").await;
//
//     let jars = Jar::query().with_owner().fetch_all(&pool).await.unwrap();
//     assert_eq!(jars.len(), 2);
//     assert!(
//         jars.iter()
//             .any(|j| j.owner.as_ref().unwrap().id == user1.id)
//     );
//     assert!(
//         jars.iter()
//             .any(|j| j.owner.as_ref().unwrap().id == user2.id)
//     );
// }
//
// #[tokio::test]
// async fn test_has_many_batch_fetch_one() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let user = insert_user(&pool, "hm@example.com", "hm").await;
//     let jar1 = insert_jar(&pool, user.id, "Jar1").await;
//     let jar2 = insert_jar(&pool, user.id, "Jar2").await;
//
//     let user_with_jars = User::query().with_jars().fetch_one(&pool).await.unwrap();
//     let jars = user_with_jars.jars.unwrap();
//     assert_eq!(jars.len(), 2);
//     assert!(jars.iter().any(|j| j.id == jar1.id));
//     assert!(jars.iter().any(|j| j.id == jar2.id));
// }
//
// #[tokio::test]
// async fn test_has_many_batch_fetch_all_multiple_users() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let user1 = insert_user(&pool, "u1@example.com", "u1").await;
//     let user2 = insert_user(&pool, "u2@example.com", "u2").await;
//
//     let jar1 = insert_jar(&pool, user1.id, "Jar1").await;
//     let jar2 = insert_jar(&pool, user1.id, "Jar2").await;
//     let jar3 = insert_jar(&pool, user2.id, "Jar3").await;
//
//     let users = User::query().with_jars().fetch_all(&pool).await.unwrap();
//
//     let u1 = users.iter().find(|u| u.id == user1.id).unwrap();
//     let u2 = users.iter().find(|u| u.id == user2.id).unwrap();
//
//     assert_eq!(u1.jars.as_ref().unwrap().len(), 2);
//     assert!(u1.jars.as_ref().unwrap().iter().any(|j| j.id == jar1.id));
//     assert!(u1.jars.as_ref().unwrap().iter().any(|j| j.id == jar2.id));
//
//     assert_eq!(u2.jars.as_ref().unwrap().len(), 1);
//     assert_eq!(u2.jars.as_ref().unwrap()[0].id, jar3.id);
// }
//
// #[tokio::test]
// async fn test_has_many_batch_empty_children() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     insert_user(&pool, "nochild@example.com", "nochild").await;
//
//     let user_with_jars = User::query().with_jars().fetch_one(&pool).await.unwrap();
//     assert!(user_with_jars.jars.unwrap().is_empty());
// }
//
// #[tokio::test]
// async fn test_relations_with_filtering_uses_correct_table_aliases() {
//     let pool = create_test_db().await;
//     migrate(&pool).await;
//
//     let user1 = insert_user(&pool, "u1@example.com", "u1").await;
//     let user2 = insert_user(&pool, "u2@example.com", "u2").await;
//
//     insert_jar(&pool, user1.id, "Jar1").await;
//     insert_jar(&pool, user2.id, "Jar2").await;
//
//     // This should use "user__.id" instead of just "id" when generating the WHERE clause
//     let result = User::query()
//         .with_jars()
//         .filter(User::ID.eq(user1.id))
//         .fetch_one(&pool)
//         .await
//         .unwrap();
//
//     assert_eq!(result.id, user1.id);
//     assert_eq!(result.username, "u1");
//
//     // Verify the generated SQL contains the correct table alias
//     let query_sql = User::query()
//         .with_jars()
//         .filter(User::ID.eq(user1.id))
//         .to_sql();
//
//     // The SQL should contain "user__.id" instead of just "id"
//     assert!(query_sql.contains("user__.id"), "Expected 'user__.id' alias in SQL, got: {}", query_sql);
// }
