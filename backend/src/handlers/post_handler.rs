use crate::{auth::AuthUser, db::DbPool, services::post_service};
use axum::{extract::Extension, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub id: i32,
    pub content: String,
    pub user_sub: String,
    pub image_url: String,
}

// Axum Handler
pub async fn create_post_handler(
    Extension(pool): Extension<DbPool>,
    Extension(user): Extension<AuthUser>,
    Json(payload): Json<CreatePostRequest>,
) -> Result<Json<PostResponse>, (StatusCode, String)> {
    let post = post_service::create_post(&pool, &user.sub, &payload.content)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(PostResponse {
        id: post.id,
        content: post.content,
        user_sub: post.user_sub,
        image_url: post.image_url,
    }))
}

// GET一覧
pub async fn list_posts_handler(
    Extension(pool): Extension<DbPool>,
) -> Result<Json<Vec<PostResponse>>, (StatusCode, String)> {
    let posts = post_service::list_posts(&pool)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let resp = posts
        .into_iter()
        .map(|p| PostResponse {
            id: p.id,
            content: p.content,
            user_sub: p.user_sub,
            image_url: p.image_url,
        })
        .collect();

    Ok(Json(resp))
}
