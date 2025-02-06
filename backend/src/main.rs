mod auth;
mod db;
mod routes;
use axum::{
    body::Body,
    extract::Extension,
    http::{Request as AxumRequest, Response as AxumResponse},
    middleware,
    response::IntoResponse,
    routing::get,
    Router,
};
use dotenvy::dotenv;
use lambda_http::{service_fn, Error as LambdaError, Request, Response};
use std::net::SocketAddr;
use tower::ServiceExt;

use auth::{auth_middleware, AuthUser};
use db::create_db_pool;
use routes::post_routes::post_routes;

/// 認証しない場合のハンドラ
async fn hello_handler() -> impl IntoResponse {
    "Hello from Axum on Lambda!"
}

async fn hoge_handler() -> impl IntoResponse {
    "Hoge from Axum on Lambda!"
}

/// axumで認証する場合のハンドラ
async fn protected_handler(Extension(user): Extension<AuthUser>) -> String {
    format!("Hello, sub={} / email={:?}", user.sub, user.email)
}

async fn lambda_req_to_axum_req(event: Request) -> Result<AxumRequest<Body>, LambdaError> {
    let (parts, body) = event.into_parts();
    let body_bytes = hyper::body::to_bytes(body)
        .await
        .map_err(|err| LambdaError::from(format!("Failed to read response body: {:?}", err)))?;
    let body = Body::from(body_bytes);

    let req = AxumRequest::from_parts(parts, body);
    Ok(req)
}

async fn axum_resp_to_lambda_resp(
    resp: AxumResponse<axum::body::BoxBody>,
) -> Result<Response<String>, LambdaError> {
    let (parts, body) = resp.into_parts();
    let body_bytes = hyper::body::to_bytes(body)
        .await
        .map_err(|err| LambdaError::from(format!("Failed to read response body: {:?}", err)))?;

    let body_string = String::from_utf8(body_bytes.to_vec())
        .map_err(|err| LambdaError::from(format!("UTF-8 error: {:?}", err)))?;

    Ok(Response::from_parts(parts, body_string))
}

/// Lambda ハンドラ（Axum Router をワンショットで呼ぶ）
async fn handler_lambda(event: Request, app: Router) -> Result<Response<String>, LambdaError> {
    let axum_req = lambda_req_to_axum_req(event).await?;
    let axum_resp = app
        .oneshot(axum_req)
        .await
        .map_err(|e| LambdaError::from(format!("Axum error: {:?}", e)))?;

    axum_resp_to_lambda_resp(axum_resp).await
}

/// ローカル開発用メイン関数
async fn run_local_dev(app: Router) -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Local dev server on http://{addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    dotenv().ok();

    // db
    let pool = create_db_pool().await;

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .layer(middleware::from_fn(auth_middleware))
        .nest("/posts", post_routes())
        .layer(Extension(pool))
        .route("/fuga", get(hoge_handler))
        .route("/hoge", get(hoge_handler))
        .route("/", get(hello_handler));

    if std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
        lambda_http::run(service_fn(move |event| handler_lambda(event, app.clone()))).await?;
    } else {
        run_local_dev(app)
            .await
            .map_err(|err| LambdaError::from(format!("Local server error: {:?}", err)))?;
    }
    Ok(())
}
