use sqlorm::Pool;
use sqlx::Executor;
use uuid::Uuid;

pub async fn create_test_db() -> Pool {
    let base_url = "postgres://us:pa@localhost:5432/".to_string();
    let admin_pool = Pool::connect(&base_url).await.unwrap();

    let db_name = format!("test_db_{}", Uuid::new_v4().to_string().replace("-", ""));
    admin_pool
        .execute(format!(r#"CREATE DATABASE "{}""#, db_name).as_str())
        .await
        .expect("Failed to create test database");

    let mut test_db_url = base_url.clone();
    if let Some(idx) = test_db_url.rfind('/') {
        test_db_url.replace_range(idx + 1.., &db_name);
    }

    Pool::connect(&test_db_url)
        .await
        .expect("Failed to connect to test database")
}
