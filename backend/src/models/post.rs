use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub user_sub: String,
    pub content: String,
    pub image_url: String,
    pub created_at: DateTime<Utc>,
}
