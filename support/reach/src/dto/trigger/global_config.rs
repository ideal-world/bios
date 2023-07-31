use std::collections::HashSet;

use bios_basic::rbum::dto::rbum_filer_dto::RbumItemBasicFilterReq;

use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    db::sea_orm,
    web::poem_openapi,
};

use crate::dto::*;
/// 添加 用户触达触发实例配置请求
#[derive(Debug, poem_openapi::Object, Deserialize)]
pub struct ReachTriggerGlobalConfigAddReq {
    /// 关联的触发场景id
    pub rel_reach_trigger_scene_id: String,
    /// 关联的触达通道
    pub rel_reach_channel: ReachChannelKind,
    /// 用户触达消息签名Id
    pub rel_reach_msg_signature_id: String,
    /// 用户触达消息模板Id
    pub rel_reach_msg_template_id: String,
}

#[derive(Debug, poem_openapi::Object, Default)]
pub struct ReachTriggerGlobalConfigModifyReq {
    /// 组件Id
    #[oai(validator(max_length = "512"))]
    pub id: Option<String>,

    /// 关联的触发场景id
    #[oai(validator(max_length = "512"))]
    pub rel_reach_trigger_scene_id: Option<String>,

    /// 关联的触达通道
    pub rel_reach_channel: Option<ReachChannelKind>,

    #[oai(validator(max_length = "255"))]
    /// 用户触达消息签名Id
    pub rel_reach_msg_signature_id: Option<String>,

    #[oai(validator(max_length = "255"))]
    /// 用户触达消息模板Id
    pub rel_reach_msg_template_id: Option<String>,
}

#[derive(Debug, poem_openapi::Object, Default)]
pub struct ReachTriggerGlobalConfigFilterReq {
    #[oai(flatten)]
    pub base_filter: RbumItemBasicFilterReq,
    #[oai(validator(max_length = "512"))]
    pub not_ids: HashSet<String>,

    /// 关联的触发场景id
    #[oai(validator(max_length = "512"))]
    pub rel_reach_trigger_scene_id: Option<String>,

    /// 关联的触达通道
    pub rel_reach_channel: Option<ReachChannelKind>,

    #[oai(validator(max_length = "255"))]
    /// 用户触达消息签名Id
    pub rel_reach_msg_signature_id: Option<String>,

    #[oai(validator(max_length = "255"))]
    /// 用户触达消息模板Id
    pub rel_reach_msg_template_id: Option<String>,
}

#[derive(Debug, poem_openapi::Object, Serialize, sea_orm::FromQueryResult)]
pub struct ReachTriggerGlobalConfigSummaryResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    /// 关联的触发场景id
    pub rel_reach_trigger_scene_id: String,
    /// 关联的触达通道
    pub rel_reach_channel: ReachChannelKind,
    /// 用户触达消息签名Id
    pub rel_reach_msg_signature_id: String,
    /// 用户触达消息模板Id
    pub rel_reach_msg_template_id: String,
}

#[derive(Debug, poem_openapi::Object, Serialize, sea_orm::FromQueryResult)]
pub struct ReachTriggerGlobalConfigDetailResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    /// 关联的触发场景id
    pub rel_reach_trigger_scene_id: String,
    /// 关联的触达通道
    pub rel_reach_channel: ReachChannelKind,
    /// 用户触达消息签名Id
    pub rel_reach_msg_signature_id: String,
    /// 用户触达消息模板Id
    pub rel_reach_msg_template_id: String,
}
