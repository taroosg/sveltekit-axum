use jsonwebtoken::DecodingKey;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Debug)]
pub enum AuthError {
    // JWK取得失敗など
    FetchError(String),
    // kidが見つからない
    MissingKid,
    // Base64 decodeや不正フォーマット
    InvalidKeyFormat(String),
}

/// Cognito JWK セット
#[derive(Debug, Deserialize)]
pub struct JwkSet {
    pub keys: Vec<Jwk>,
}

/// 各キー情報
#[derive(Debug, Deserialize, Clone)]
pub struct Jwk {
    // pub alg: String,
    // pub kty: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub n: String,
    pub e: String,
    pub kid: String,
    // x5t等は省略
}

/// キャッシュに保持するマップ
#[derive(Debug)]
pub struct JwkCache {
    pub map: HashMap<String, Jwk>,
}

/// once_cell + tokio::RwLockでスレッド安全
pub static JWK_CACHE: Lazy<Arc<RwLock<Option<JwkCache>>>> =
    Lazy::new(|| Arc::new(RwLock::new(None)));

/// JWKS を取得してJwkCacheを作る
async fn fetch_jwks(region: &str, user_pool_id: &str) -> Result<JwkCache, AuthError> {
    let url =
        format!("https://cognito-idp.{region}.amazonaws.com/{user_pool_id}/.well-known/jwks.json");
    let resp = reqwest::get(url)
        .await
        .map_err(|err| AuthError::FetchError(format!("reqwest error: {}", err)))?;
    let jwk_set: JwkSet = resp
        .json()
        .await
        .map_err(|err| AuthError::FetchError(format!("json parse error: {}", err)))?;

    let mut map = HashMap::new();
    for jwk in jwk_set.keys {
        map.insert(jwk.kid.clone(), jwk);
    }
    Ok(JwkCache { map })
}

/// kid に該当する公開鍵をDecodingKeyに変換
fn jwk_to_decoding_key(jwk: &Jwk) -> Result<DecodingKey, AuthError> {
    DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|_| AuthError::InvalidKeyFormat("DecodingKey creation failed".to_string()))
}

/// kid に対応する DecodingKey を返す (キャッシュ利用)
pub async fn get_decoding_key_for_kid(
    kid: &str,
    region: &str,
    user_pool_id: &str,
) -> Result<DecodingKey, AuthError> {
    {
        // 1) キャッシュを読む
        let cache = JWK_CACHE.read().await;
        if let Some(ref c) = *cache {
            if let Some(jwk) = c.map.get(kid) {
                return jwk_to_decoding_key(jwk);
            }
        }
    }
    // 2) キャッシュに無い → fetch
    let fetched = fetch_jwks(region, user_pool_id).await?;
    {
        let mut wcache = JWK_CACHE.write().await;
        *wcache = Some(fetched);
    }
    // 3) 再読み込み
    let cache = JWK_CACHE.read().await;
    if let Some(ref c) = *cache {
        if let Some(jwk) = c.map.get(kid) {
            return jwk_to_decoding_key(jwk);
        }
    }
    Err(AuthError::MissingKid)
}
