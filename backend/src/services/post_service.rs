use crate::db::DbPool;
use crate::models::post::Post;
use sqlx::Error;

pub async fn create_post(pool: &DbPool, user_sub: &str, content: &str) -> Result<Post, Error> {
    let image_url = "dummy-image.jpg";
    let row = sqlx::query_as!(
        Post,
        r#"
        INSERT INTO posts (user_sub, content, image_url)
        VALUES ($1, $2, $3)
        RETURNING id, user_sub, content, image_url, created_at
        "#,
        user_sub,
        content,
        image_url
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn list_posts(pool: &DbPool) -> Result<Vec<Post>, Error> {
    sqlx::query_as!(
        Post,
        r#"SELECT id, user_sub, content, image_url, created_at FROM posts ORDER BY created_at DESC"#
    )
    .fetch_all(pool)
    .await
}
