use axum::routing::{get, post};
use axum::Router;

use crate::handlers::post_handlers::{create_post_controller, list_posts_controller};

pub fn post_routes() -> Router {
    Router::new().route(
        "/posts",
        post(create_post_controller).get(list_posts_controller),
    )
}
