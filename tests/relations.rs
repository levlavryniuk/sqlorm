use common::entities::{DonationExecutor, JarExecutor, UserExecutor};
mod common;
use common::entities::{JarRelations, UserRelations};
use uuid::Uuid;

use common::create_clean_db;
use common::entities::{Donation, Jar, User};

async fn setup_test_data(pool: &sqlorm::Pool) -> (User, User, Jar, Jar, Donation, Donation) {
    let user1 = User::test_user("owner1@example.com", "owner1")
        .save(pool)
        .await
        .expect("Failed to save user1");

    let user2 = User::test_user("owner2@example.com", "owner2")
        .save(pool)
        .await
        .expect("Failed to save user2");

    let mut jar1 = Jar::test_jar(user1.id, "jar1");
    jar1.title = "User1's Jar".to_string();
    let jar1 = jar1.save(pool).await.expect("Failed to save jar1");

    let mut jar2 = Jar::test_jar(user2.id, "jar2");
    jar2.title = "User2's Jar".to_string();
    let jar2 = jar2.save(pool).await.expect("Failed to save jar2");

    let donation1 = Donation::test_donation(jar1.id, user2.id, 25.0)
        .save(pool)
        .await
        .expect("Failed to save donation1");

    let donation2 = Donation::test_donation(jar2.id, user1.id, 50.0)
        .save(pool)
        .await
        .expect("Failed to save donation2");

    (user1, user2, jar1, jar2, donation1, donation2)
}

#[tokio::test]
async fn test_belongs_to_lazy_loading() {
    let pool = create_clean_db().await;
    let (user1, _user2, jar1, _jar2, donation1, _donation2) = setup_test_data(&pool).await;

    let owner = jar1
        .owner(&pool)
        .await
        .expect("Failed to load owner")
        .expect("Owner not found");
    assert_eq!(owner.id, user1.id);
    assert_eq!(owner.username, "owner1");

    let jar = donation1
        .jar(&pool)
        .await
        .expect("Failed to load jar")
        .expect("Jar not found");
    assert_eq!(jar.id, jar1.id);
    assert_eq!(jar.title, "User1's Jar");

    let payer = donation1
        .payer(&pool)
        .await
        .expect("Failed to load payer")
        .expect("Payer not found");
    assert_eq!(payer.id, donation1.payer_id);
}

#[tokio::test]
async fn test_belongs_to_eager_loading() {
    let pool = create_clean_db().await;
    let (_user1, _user2, _jar1, _jar2, _donation1, _donation2) = setup_test_data(&pool).await;

    let jar_with_owner = Jar::query()
        .with_owner()
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch jar with owner");

    let owner = jar_with_owner.owner.expect("Owner should be loaded");
    assert_eq!(owner.id, jar_with_owner.owner_id);

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

    let user1_jars = user1.jars(&pool).await.expect("Failed to load user1 jars");
    assert_eq!(user1_jars.len(), 1);
    assert_eq!(user1_jars[0].owner_id, user1.id);
    assert_eq!(user1_jars[0].title, "User1's Jar");

    let user2_jars = user2.jars(&pool).await.expect("Failed to load user2 jars");
    assert_eq!(user2_jars.len(), 1);
    assert_eq!(user2_jars[0].owner_id, user2.id);
    assert_eq!(user2_jars[0].title, "User2's Jar");

    let user1_donations = user1
        .payed_donations(&pool)
        .await
        .expect("Failed to load user1 donations");
    assert_eq!(user1_donations.len(), 1);
    assert_eq!(user1_donations[0].payer_id, user1.id);
    assert_eq!(user1_donations[0].amount, 50.0);

    let user2_donations = user2
        .payed_donations(&pool)
        .await
        .expect("Failed to load user2 donations");
    assert_eq!(user2_donations.len(), 1);
    assert_eq!(user2_donations[0].payer_id, user2.id);
    assert_eq!(user2_donations[0].amount, 25.0);
}

#[tokio::test]
async fn test_has_many_eager_loading() {
    let pool = create_clean_db().await;
    let (_user1, _user2, jar1, _jar2, _donation1, _donation2) = setup_test_data(&pool).await;

    let user_with_jars = User::query()
        .with_jars()
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch user with jars");

    let jars = user_with_jars.jars.expect("Jars should be loaded");
    assert_eq!(jars.len(), 1);
    assert_eq!(jars[0].owner_id, user_with_jars.id);

    let jar_with_donations = Jar::query()
        .filter(Jar::ID.eq(jar1.id))
        .with_donations()
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch jar with donations");

    let donations = jar_with_donations
        .donations
        .expect("Donations should be loaded");
    assert_eq!(donations.len(), 1);
    assert_eq!(donations[0].jar_id, jar1.id);
    assert_eq!(donations[0].amount, 25.0);
}

#[tokio::test]
async fn test_has_many_empty_relations() {
    let pool = create_clean_db().await;

    let user = User::test_user("lonely@example.com", "lonely")
        .save(&pool)
        .await
        .expect("Failed to save user");

    let jars = user.jars(&pool).await.expect("Failed to load jars");
    assert!(jars.is_empty());

    let donations = user
        .payed_donations(&pool)
        .await
        .expect("Failed to load donations");
    assert!(donations.is_empty());

    let user_with_jars = User::query()
        .filter(User::ID.eq(user.id))
        .with_jars()
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch user with jars");

    let jars = user_with_jars.jars.expect("Jars should be loaded");
    assert!(jars.is_empty());
}
