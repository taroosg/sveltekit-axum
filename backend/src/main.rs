use axum::extract::Extension;
use axum::{body::Body, response::IntoResponse, routing::get, Router};
use lambda_http::{service_fn, Error, Request, RequestExt, Response};
use std::net::SocketAddr;

async fn hello_handler() -> impl IntoResponse {
    "Hello from Axum on Lambda!"
}

// For local dev
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Distinguish local dev vs Lambda
    if std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
        lambda_http::run(service_fn(handler_lambda)).await?;
    } else {
        // local dev: run normal Axum server
        let app = Router::new().route("/", get(hello_handler));
        let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
        println!("Listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;
    }
    Ok(())
}

// Lambda request => Axum router
async fn handler_lambda(event: Request) -> Result<Response<String>, Error> {
    // Create Axum Router each invocation or keep it static
    let _app: Router<(), Body> = Router::new().route("/", get(hello_handler));
    // For complex apps, you'd parse path, method, etc
    // but here we keep it minimal
    let resp_body = "Hello from Axum on Lambda (via APIGW)!";
    let resp = Response::builder()
        .status(200)
        .body(resp_body.into())
        .unwrap();
    Ok(resp)
}
