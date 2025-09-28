use sqlorm::StatementExecutor;
mod common;

use common::create_clean_db;
use common::entities::{Jar, JarExecutor, User, UserExecutor};

#[tokio::test]
async fn test_user_soft_delete_method() {
    let pool = create_clean_db().await;
    let user = User::test_user("soft@example.com", "softuser")
        .save(&pool)
        .await
        .unwrap();
    let id = user.id;

    user.delete().execute(&pool).await.unwrap();

    let db_user: User = User::query()
        .filter(User::ID.eq(id))
        .fetch_one(&pool)
        .await
        .unwrap();

    assert!(
        db_user.deleted_at.is_some(),
        "Soft delete should set deleted_at"
    );
}

#[tokio::test]
async fn test_jar_hard_delete_method() {
    let pool = create_clean_db().await;
    let user = User::test_user("jarowner@example.com", "jarowner")
        .save(&pool)
        .await
        .unwrap();

    let jar = Jar::test_jar(user.id.clone(), "hardjar")
        .save(&pool)
        .await
        .unwrap();

    let id = jar.id;

    jar.delete().execute(&pool).await.unwrap();

    let maybe_jar: Option<Jar> = Jar::query()
        .filter(Jar::ID.eq(id))
        .fetch_optional(&pool)
        .await
        .unwrap();

    assert!(
        maybe_jar.is_none(),
        "Hard delete should physically remove Jar"
    );
}
