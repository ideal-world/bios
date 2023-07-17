use serde::Deserialize;
use tardis::web::poem_openapi;

use crate::dto::*;

/// 添加或编辑用户触达触发实例聚合配置请求
#[derive(Debug, poem_openapi::Object, Deserialize)]
pub struct ReachTriggerInstanceConfigAddOrModifyAggReq {
    pub instance_config: Vec<ReachTriggerInstanceConfigAddOrModifyReq>,
}

#[derive(Debug, poem_openapi::Object, Deserialize)]
pub struct ReachTriggerInstanceConfigAddOrModifyReq {
    #[oai(validator(max_length = "512"))]
    /// 关联的触发场景id
    pub rel_reach_trigger_scene_id: String,
    /// 关联的触达通道
    pub rel_reach_channel: ReachChannelKind,
    #[oai(validator(max_length = "512"))]
    /// 关联资源项id
    pub rel_item_id: String,
    #[oai(validator(max_length = "255"))]
    /// 接收组编码
    pub receive_group_code: String,
    #[oai(validator(max_length = "255"))]
    /// 接收组名称
    pub receive_group_name: String,
    /// 是否删除
    pub delete_kind: bool,
}

impl From<ReachTriggerInstanceConfigAddOrModifyReq> for ReachTriggerInstanceConfigAddReq {
    fn from(val: ReachTriggerInstanceConfigAddOrModifyReq) -> Self {
        ReachTriggerInstanceConfigAddReq {
            rel_reach_trigger_scene_id: val.rel_reach_trigger_scene_id,
            rel_reach_channel: val.rel_reach_channel,
            rel_item_id: val.rel_item_id,
            receive_group_code: val.receive_group_code,
            receive_group_name: val.receive_group_name,
        }
    }
}
