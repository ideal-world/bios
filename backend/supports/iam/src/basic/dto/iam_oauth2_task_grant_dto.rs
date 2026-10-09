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
