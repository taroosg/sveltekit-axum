use axum::{
    body::Body,
    extract::Extension,
    http::{Request as AxumRequest, Response as AxumResponse},
    middleware,
    response::IntoResponse,
    routing::get,
    Router,
};
use lambda_http::{service_fn, Error as LambdaError, Request, Response};
use std::net::SocketAddr;
use tower::ServiceExt;

mod auth;
use auth::{auth_middleware, AuthUser};

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

fn create_app() -> Router {
    Router::new()
        .route("/protected", get(protected_handler))
        .layer(middleware::from_fn(auth_middleware))
        .route("/fuga", get(hoge_handler))
        .route("/hoge", get(hoge_handler))
        .route("/", get(hello_handler))
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
async fn handler_lambda(event: Request) -> Result<Response<String>, LambdaError> {
    let app = create_app();

    let axum_req = lambda_req_to_axum_req(event).await?;
    let axum_resp = app
        .oneshot(axum_req)
        .await
        .map_err(|e| LambdaError::from(format!("Axum error: {:?}", e)))?;

    axum_resp_to_lambda_resp(axum_resp).await
}

/// ローカル開発用メイン関数
async fn run_local_dev() -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// エントリーポイント
#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    // AWS Lambda 環境では AWS_LAMBDA_FUNCTION_NAME が設定される
    if std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
        // Lambda 本番動作: lambda_http::run(service_fn(...))
        lambda_http::run(service_fn(handler_lambda)).await?;
    } else {
        // ローカル開発
        run_local_dev()
            .await
            .map_err(|err| LambdaError::from(format!("Local server error: {:?}", err)))?;
    }
    Ok(())
}
