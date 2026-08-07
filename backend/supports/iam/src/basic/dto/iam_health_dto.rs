use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::serde_json::Value;
use tardis::web::poem_openapi;

/// Health Check Response (Spring Boot Actuator style)
/// 健康检查响应（Spring Boot Actuator 风格）
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamHealthResp {
    /// Overall status: UP / DOWN
    /// 整体状态：UP / DOWN
    pub status: String,
    /// Health groups
    /// 健康分组
    pub groups: Vec<String>,
    /// Component status
    /// 各组件状态
    pub components: HashMap<String, IamHealthComponentResp>,
}

/// Component Health Status
/// 组件健康状态
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamHealthComponentResp {
    /// Component status: UP / DOWN
    /// 组件状态：UP / DOWN
    pub status: String,
    /// Component details (optional)
    /// 组件详情（可选）
    #[oai(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}
