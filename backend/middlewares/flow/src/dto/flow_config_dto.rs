use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowConfigModifyReq {
    pub code: String,
    pub value: String,
}

// 评审相关配置
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowRootConfigResp {
    pub code: String,
    pub label: String,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
#[serde(rename_all = "camelCase")]
pub struct FlowReviewConfigLabelResp {
    pub origin_status: Vec<String>,
    pub pass_status: String,
    pub unpass_status: String,
}
