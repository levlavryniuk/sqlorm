use chrono::{DateTime, Utc};
use criterion::{Criterion, criterion_group, criterion_main};
use serde::{Deserialize, Serialize};
use sqlorm::prelude::*;
use sqlorm::table;
use std::hint::black_box;
use std::sync::atomic::{AtomicU32, Ordering};

#[table(name = "users")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    #[sql(pk)]
    pub id: i64,
    pub email: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub bio: Option<String>,
    #[sql(timestamp(created_at, chrono::Utc::now()))]
    pub created_at: DateTime<Utc>,
    #[sql(timestamp(updated_at, chrono::Utc::now()))]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct RawUser {
    id: i64,
    email: String,
    username: String,
    first_name: String,
    last_name: String,
    bio: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    fn new_test_user(index: u32) -> Self {
        Self {
            email: format!("user{}@benchmark.com", index),
            username: format!("user{}", index),
            first_name: "Benchmark".to_string(),
            last_name: "User".to_string(),
            bio: Some("Performance testing user".to_string()),
            ..Default::default()
        }
    }
}

struct TestData {
    user_ids: Vec<i64>,
}

async fn setup_test_pool() -> (sqlorm::Pool, TestData) {
    let pool = sqlorm::Pool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite database");

    sqlx::query(
        r#"
        CREATE TABLE "users" (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT NOT NULL,
            username TEXT NOT NULL,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            bio TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    let mut user_ids = Vec::new();
    for i in 0..10 {
        let user = User::new_test_user(i)
            .save(&pool)
            .await
            .expect("Failed to seed user");
        user_ids.push(user.id);
    }

    (pool, TestData { user_ids })
}

fn crud_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create async runtime");
    let (pool, test_data) = rt.block_on(setup_test_pool());

    let mut g_insert = c.benchmark_group("insert");
    g_insert.sample_size(100);
    {
        let counter = AtomicU32::new(1000);
        g_insert.bench_function("sqlorm_insert", |b| {
            b.to_async(&rt).iter(|| async {
                let id = counter.fetch_add(1, Ordering::Relaxed);
                let user = black_box(User::new_test_user(id));
                black_box(user.save(&pool).await.expect("save failed"))
            })
        });

        let counter = AtomicU32::new(2000);
        g_insert.bench_function("raw_sqlx_insert", |b| {
            b.to_async(&rt).iter(|| async {
                let id = counter.fetch_add(1, Ordering::Relaxed);
                let email = black_box(format!("user{}@benchmark.com", id));
                let username = black_box(format!("user{}", id));
                let now = chrono::Utc::now();

                black_box(
                    sqlx::query(
                        "INSERT INTO users (email, username, first_name, last_name, bio, created_at, updated_at) 
                         VALUES (?, ?, ?, ?, ?, ?, ?)"
                    )
                    .bind(&email)
                    .bind(&username)
                    .bind("Benchmark")
                    .bind("User")
                    .bind("Performance testing user")
                    .bind(&now)
                    .bind(&now)
                    .execute(&pool)
                    .await
                    .expect("insert failed"),
                )
            })
        });
    }
    g_insert.finish();

    let mut g_find = c.benchmark_group("find_by_id");
    {
        let user_id = test_data.user_ids[0];
        g_find.bench_function("sqlorm_find_by_id", |b| {
            b.to_async(&rt).iter(|| async {
                black_box(
                    User::query()
                        .filter(User::ID.eq(user_id))
                        .fetch_one(&pool)
                        .await
                        .expect("find failed"),
                )
            })
        });

        g_find.bench_function("raw_sqlx_find_by_id", |b| {
            b.to_async(&rt).iter(|| async {
                black_box(
                    sqlx::query_as::<_, RawUser>("SELECT * FROM users WHERE id = ?")
                        .bind(user_id)
                        .fetch_one(&pool)
                        .await
                        .expect("find failed"),
                )
            })
        });
    }
    g_find.finish();

    let mut g_update = c.benchmark_group("update");
    {
        let update_user_id = test_data.user_ids[1];

        let counter = AtomicU32::new(5000);
        g_update.bench_function("sqlorm_update", |b| {
            b.to_async(&rt).iter(|| async {
                let suffix = counter.fetch_add(1, Ordering::Relaxed);
                let mut user = User::query()
                    .filter(User::ID.eq(update_user_id))
                    .fetch_one(&pool)
                    .await
                    .expect("fetch failed");

                user.bio = Some(black_box(format!("Updated bio {}", suffix)));
                black_box(user.save(&pool).await.expect("update failed"))
            })
        });

        let counter = AtomicU32::new(6000);
        g_update.bench_function("raw_sqlx_update", |b| {
            b.to_async(&rt).iter(|| async {
                let suffix = counter.fetch_add(1, Ordering::Relaxed);
                let bio = black_box(format!("Updated bio {}", suffix));
                let now = chrono::Utc::now();

                black_box(
                    sqlx::query("UPDATE users SET bio = ?, updated_at = ? WHERE id = ?")
                        .bind(&bio)
                        .bind(&now)
                        .bind(update_user_id)
                        .execute(&pool)
                        .await
                        .expect("update failed"),
                )
            })
        });
    }
    g_update.finish();

    let mut g_filter = c.benchmark_group("filter_query");
    g_filter.sample_size(20);
    {
        g_filter.bench_function("sqlorm_query_filter", |b| {
            b.to_async(&rt).iter(|| async {
                black_box(
                    User::query()
                        .filter(User::ID.gt(5))
                        .filter(User::FIRST_NAME.eq("Benchmark".to_string()))
                        .fetch_all(&pool)
                        .await
                        .expect("filter failed"),
                )
            })
        });

        g_filter.bench_function("raw_sqlx_query_filter", |b| {
            b.to_async(&rt).iter(|| async {
                black_box(
                    sqlx::query_as::<_, RawUser>(
                        "SELECT * FROM users WHERE id > ? AND first_name = ?",
                    )
                    .bind(5)
                    .bind("Benchmark")
                    .fetch_all(&pool)
                    .await
                    .expect("filter failed"),
                )
            })
        });
    }
    g_filter.finish();
}

criterion_group!(benches, crud_benchmark);
criterion_main!(benches);
