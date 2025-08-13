use sqlorm_macros::DeriveSchema;
use sqlx::PgPool;
async fn get_pool() -> sqlx::PgPool {
    PgPool::connect("postgres://test:test").await.unwrap()
}

#[derive(Debug, DeriveSchema)]
#[table(name = "user")]
struct User {
    #[sql(pk)]
    id: String,
    name: String,
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[tokio::test]
    async fn user() {
        let mut p = get_pool().await;
        let u = User {
            id: "3".into(),
            name: "Lev".into(),
        };
        assert_eq!(User::table_name(), "user");
        assert_eq!(&u.name, "Lev");
        let u = User::find_by_id(&mut p, "1".to_string());
    }
}
