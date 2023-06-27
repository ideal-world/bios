use std::time::*;

use serde::{Deserialize, Serialize};

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::types::*;

use super::conf_namespace_dto::NamespaceAttribute;
#[derive(Debug, Serialize, Deserialize, poem_openapi::Object)]
pub struct NacosResponse<T: Type + ParseFromJSON + ToJSON> {
    code: u16,
    message: Option<String>,
    data: T,
}

impl<T: Type + ParseFromJSON + ToJSON> NacosResponse<T> {
    pub const fn ok(data: T) -> Self {
        Self { code: 200, message: None, data }
    }
}

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
            sub: String::from(user),
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
    #[oai(rename = "accessToken")]
    pub access_token: String,
    #[oai(rename = "tokenTtl")]
    pub token_ttl: u32,
    #[oai(rename = "globalAdmin")]
    pub global_admin: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct PublishConfigForm {
    pub content: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[allow(non_snake_case)]
pub struct NacosCreateNamespaceRequest {
    customNamespaceId: String,
    namespaceName: String,
    namespaceDesc: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[allow(non_snake_case)]
pub struct NacosEditNamespaceRequest {
    namespace: String,
    namespaceShowName: String,
    namespaceDesc: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
#[allow(non_snake_case)]
pub struct NacosDeleteNamespaceRequest {
    pub(crate) namespaceId: String,
}

impl From<NacosCreateNamespaceRequest> for NamespaceAttribute {
    fn from(value: NacosCreateNamespaceRequest) -> Self {
        Self {
            namespace: value.customNamespaceId,
            namespace_show_name: value.namespaceName,
            namespace_desc: value.namespaceDesc,
        }
    }
}

impl From<NacosEditNamespaceRequest> for NamespaceAttribute {
    fn from(value: NacosEditNamespaceRequest) -> Self {
        Self {
            namespace: value.namespace,
            namespace_show_name: value.namespaceShowName,
            namespace_desc: value.namespaceDesc,
        }
    }
}
