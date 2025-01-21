use sqlx::{Pool, Postgres};
use std::env;

pub type DbPool = Pool<Postgres>;

pub async fn create_db_pool() -> DbPool {
    // 環境変数 "DATABASE_URL" に "postgres://user:pass@host:5432/dbname"
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // DBが存在しない場合、作る(オプション)
    // if !Postgres::database_exists(&db_url).await.unwrap() {
    //     Postgres::create_database(&db_url).await.unwrap();
    // }

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to Postgres");

    // マイグレーションを実行
    // sqlx::migrate!("./migrations")
    //     .run(&pool)
    //     .await
    //     .expect("Failed to run SQLx migrations");

    pool
}
