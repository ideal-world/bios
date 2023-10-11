use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

use super::ReachTriggerGlobalConfigAddReq;

/// 添加或编辑用户触达触发全局聚合配置请求
#[derive(Debug, poem_openapi::Object, Serialize, Deserialize)]
pub struct ReachTriggerGlobalConfigAddOrModifyAggReq {
    pub global_config: Vec<ReachTriggerGlobalConfigAddReq>,
}
