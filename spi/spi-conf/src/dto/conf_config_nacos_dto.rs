use std::time::*;

use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

#[derive(Debug, Serialize, Deserialize)]
pub struct NacosJwtClaim {
    pub exp: u64,
    pub sub: String,
}

impl NacosJwtClaim {
    pub fn gen(ttl: u64, user: &str) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("invalid system time cause by time travel").as_secs();
        Self {
            exp: now + ttl,
            sub: String::from(user)
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[serde(rename = "camelCase")]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[serde(rename = "camelCase")]
pub struct LoginResponse {
    #[oai(rename="accessToken")]
    pub access_token: String,
    #[oai(rename="tokenTtl")]
    pub token_ttl: u32,
    #[oai(rename="globalAdmin")]
    pub global_admin: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct PublishConfigForm {
    pub content: String
}