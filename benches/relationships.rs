use chrono::{DateTime, Utc};
use criterion::{Criterion, criterion_group, criterion_main};
use serde::{Deserialize, Serialize};
use sqlorm::prelude::*;
use sqlorm::table;
use std::hint::black_box;

#[table(name = "authors")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Author {
    #[sql(pk)]
    #[sql(relation(has_many -> Post, relation = "posts", on = author_id))]
    pub id: i64,
    pub name: String,
    #[sql(timestamp(created_at, chrono::Utc::now()))]
    pub created_at: DateTime<Utc>,
    #[sql(timestamp(updated_at, chrono::Utc::now()))]
    pub updated_at: DateTime<Utc>,
}

#[table(name = "posts")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Post {
    #[sql(pk)]
    pub id: i64,
    pub title: String,
    pub content: String,
    #[sql(relation(belongs_to -> Author, relation = "author", on = id))]
    pub author_id: i64,
    #[sql(timestamp(created_at, chrono::Utc::now()))]
    pub created_at: DateTime<Utc>,
    #[sql(timestamp(updated_at, chrono::Utc::now()))]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct RawAuthor {
    id: i64,
    name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct RawPost {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct RawAuthorPost {
    author_id: i64,
    author_name: String,
    post_id: i64,
    post_title: String,
    post_content: String,
}

struct TestData {
    author_id: i64,
    post_id: i64,
}

impl Author {
    fn new_test_author(index: u32) -> Self {
        Self {
            name: format!("Author {}", index),
            ..Default::default()
        }
    }
}

impl Post {
    fn new_test_post(author_id: i64, index: u32) -> Self {
        Self {
            title: format!("Post {}", index),
            content: format!("Content for post {}", index),
            author_id,
            ..Default::default()
        }
    }
}

async fn setup_relationship_pool() -> (sqlorm::Pool, TestData) {
    let pool = sqlorm::Pool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite database");

    sqlx::query(
        r#"
        CREATE TABLE "authors" (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create authors table");

    sqlx::query(
        r#"
        CREATE TABLE "posts" (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            author_id INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create posts table");

    let author = Author::new_test_author(1)
        .save(&pool)
        .await
        .expect("Failed to create test author");

    let mut post_id = 0;
    for i in 0..3 {
        let post = Post::new_test_post(author.id, i)
            .save(&pool)
            .await
            .expect("Failed to create test post");
        if i == 0 {
            post_id = post.id;
        }
    }

    (
        pool,
        TestData {
            author_id: author.id,
            post_id,
        },
    )
}

fn relationship_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create async runtime");
    let (pool, test_data) = rt.block_on(setup_relationship_pool());

    // ----------------------
    // Author -> Posts
    // ----------------------
    let mut g_author_posts_lazy = c.benchmark_group("author_posts_lazy");
    g_author_posts_lazy.sample_size(300);

    g_author_posts_lazy.bench_function("sqlorm_lazy_load_posts", |b| {
        b.to_async(&rt).iter(|| async {
            let author = Author::query()
                .filter(Author::ID.eq(test_data.author_id))
                .fetch_one(&pool)
                .await
                .expect("Failed to find author");

            black_box(author.posts(&pool).await.expect("Failed to load posts"))
        })
    });

    g_author_posts_lazy.bench_function("raw_sqlx_lazy_load_posts", |b| {
        b.to_async(&rt).iter(|| async {
            let _author = sqlx::query_as::<_, RawAuthor>("SELECT * FROM authors WHERE id = ?")
                .bind(test_data.author_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to find author");

            black_box(
                sqlx::query_as::<_, RawPost>("SELECT * FROM posts WHERE author_id = ?")
                    .bind(test_data.author_id)
                    .fetch_all(&pool)
                    .await
                    .expect("Failed to load posts"),
            )
        })
    });
    g_author_posts_lazy.finish();

    let mut g_author_posts_eager = c.benchmark_group("author_posts_eager");

    g_author_posts_eager.bench_function("sqlorm_eager_load_posts", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(
                Author::query()
                    .filter(Author::ID.eq(test_data.author_id))
                    .with_posts()
                    .fetch_one(&pool)
                    .await
                    .expect("Failed to fetch author with posts"),
            )
        })
    });

    g_author_posts_eager.bench_function("raw_sqlx_eager_load_posts", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(
                sqlx::query_as::<_, RawAuthorPost>(
                    "SELECT a.id as author_id, a.name as author_name, \
                            p.id as post_id, p.title as post_title, p.content as post_content \
                     FROM authors a \
                     JOIN posts p ON a.id = p.author_id \
                     WHERE a.id = ?",
                )
                .bind(test_data.author_id)
                .fetch_all(&pool)
                .await
                .expect("Failed to execute join query"),
            )
        })
    });
    g_author_posts_eager.finish();

    // ----------------------
    // Post -> Author
    // ----------------------
    let mut g_post_author_lazy = c.benchmark_group("post_author_lazy");
    g_post_author_lazy.sample_size(300);

    g_post_author_lazy.bench_function("sqlorm_lazy_load_author", |b| {
        b.to_async(&rt).iter(|| async {
            let post = Post::query()
                .filter(Post::ID.eq(test_data.post_id))
                .fetch_one(&pool)
                .await
                .expect("Failed to find post");

            black_box(
                post.author(&pool)
                    .await
                    .expect("Failed to load author")
                    .expect("Author should exist"),
            )
        })
    });

    g_post_author_lazy.bench_function("raw_sqlx_lazy_load_author", |b| {
        b.to_async(&rt).iter(|| async {
            let post = sqlx::query_as::<_, RawPost>("SELECT * FROM posts WHERE id = ?")
                .bind(test_data.post_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to find post");

            black_box(
                sqlx::query_as::<_, RawAuthor>("SELECT * FROM authors WHERE id = ?")
                    .bind(post.author_id)
                    .fetch_one(&pool)
                    .await
                    .expect("Failed to load author"),
            )
        })
    });
    g_post_author_lazy.finish();

    let mut g_post_author_eager = c.benchmark_group("post_author_eager");

    g_post_author_eager.bench_function("sqlorm_eager_load_author", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(
                Post::query()
                    .filter(Post::ID.eq(test_data.post_id))
                    .with_author()
                    .fetch_one(&pool)
                    .await
                    .expect("Failed to fetch post with author"),
            )
        })
    });

    g_post_author_eager.bench_function("raw_sqlx_eager_load_author", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(
                sqlx::query_as::<_, RawAuthorPost>(
                    "SELECT a.id as author_id, a.name as author_name, \
                            p.id as post_id, p.title as post_title, p.content as post_content \
                     FROM posts p \
                     JOIN authors a ON p.author_id = a.id \
                     WHERE p.id = ?",
                )
                .bind(test_data.post_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to execute join query"),
            )
        })
    });
    g_post_author_eager.finish();
}

criterion_group!(benches, relationship_benchmark);
criterion_main!(benches);
