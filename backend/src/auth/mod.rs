pub mod jwk;
use crate::auth::jwk::get_decoding_key_for_kid;
use axum::{http::StatusCode, middleware::Next, response::Response};
use dotenvy::dotenv;
use jsonwebtoken::{decode, Algorithm, TokenData, Validation};
use serde::Deserialize;
use std::env;

/// Cognitoのトークンに含まれる代表的クレームの例
#[derive(Debug, Deserialize)]
pub struct CognitoClaims {
    pub sub: String,
    pub email: Option<String>,
    // pub exp: usize,
    // pub iss: String,
}

/// リクエスト拡張に持たせるユーザ情報
#[derive(Clone)]
pub struct AuthUser {
    pub sub: String,
    pub email: Option<String>,
}

pub fn extract_bearer<B>(req: &axum::http::Request<B>) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .filter(|val| val.starts_with("Bearer "))
        .map(|val| val["Bearer ".len()..].to_string())
}

/// JWT検証用のミドルウェア
pub async fn auth_middleware<B>(
    mut req: axum::http::Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    dotenv().ok();

    let token = extract_bearer(&req).ok_or(StatusCode::UNAUTHORIZED)?;

    // 1) ヘッダ部(kid)を取り出す
    let header = jsonwebtoken::decode_header(&token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let kid = header.kid.ok_or(StatusCode::UNAUTHORIZED)?;

    // 2) デコードキー取得
    let region = env::var("COGNITO_REGION").unwrap_or("us-east-1".to_string());
    let user_pool_id = env::var("COGNITO_USER_POOL_ID").expect("UserPoolId needed");

    let decoding_key = get_decoding_key_for_kid(&kid, &region, &user_pool_id)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 3) JWT decode
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[format!(
        "https://cognito-idp.{region}.amazonaws.com/{user_pool_id}"
    )]);
    validation
        .set_audience(&[env::var("COGNITO_USER_POOL_CLIENT_ID").expect("AppClientId needed")]);

    let token_data: TokenData<CognitoClaims> =
        decode(&token, &decoding_key, &validation).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let claims = token_data.claims;

    let user = AuthUser {
        sub: claims.sub,
        email: claims.email,
    };

    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}

// ユーザを取り出すためのヘルパー
// pub async fn get_auth_user(
//     axum::extract::Extension(user): axum::extract::Extension<AuthUser>,
// ) -> AuthUser {
//     user
// }
