use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

#[derive(Clone, Debug, Serialize, Deserialize, poem_openapi::Object)]
pub struct IamOAuth2TaskGrantCreateReq {
    pub task_id: String,
    pub requires_external_oauth: bool,
    pub expected_grant_id: Option<String>,
    #[serde(default)]
    pub takeover: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, poem_openapi::Object)]
pub struct IamOAuth2TaskGrantRef {
    pub grant_id: String,
    pub account_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, poem_openapi::Object)]
pub struct IamOAuth2TaskGrantExchangeReq {
    pub grant_id: String,
    pub task_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, poem_openapi::Object)]
pub struct IamOAuth2TaskGrantToken {
    pub access_token: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, poem_openapi::Object)]
pub struct IamOAuth2TaskGrantCurrentReq {
    pub task_id: String,
}

/// 仅由受信后台服务兑换，授权用户从持久化 grant 读取。
#[derive(Clone, Debug, Serialize, Deserialize, poem_openapi::Object)]
pub struct IamOAuth2TaskRuntimeReq {
    pub grant_id: String,
    pub task_id: String,
    pub own_paths: String,
}

/// 本次后台运行的短期本地凭据；不包含 refresh token。
#[derive(Serialize, Deserialize, poem_openapi::Object)]
pub struct IamOAuth2TaskRuntimeResp {
    pub account_id: String,
    pub own_paths: String,
    pub bios_token: String,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
}
