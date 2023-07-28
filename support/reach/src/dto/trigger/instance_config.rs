use bios_basic::rbum::dto::rbum_filer_dto::RbumItemBasicFilterReq;


use serde::Serialize;
use tardis::{
    chrono::{DateTime, Utc},
    db::sea_orm,
    web::poem_openapi,
};

use crate::dto::*;

/// 添加 用户触达触发实例配置请求
#[derive(Debug, poem_openapi::Object)]
pub struct ReachTriggerInstanceConfigAddReq {
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
}

#[derive(Debug, poem_openapi::Object)]
pub struct ReachTriggerInstanceConfigModifyReq {
    #[oai(validator(max_length = "512"))]
    /// 关联的触发场景id
    pub rel_reach_trigger_scene_id: Option<String>,
    /// 关联的触达通道
    pub rel_reach_channel: Option<ReachChannelKind>,
    #[oai(validator(max_length = "512"))]
    /// 关联资源项id
    pub rel_item_id: Option<String>,
    #[oai(validator(max_length = "255"))]
    /// 接收组编码
    pub receive_group_code: Option<String>,
    #[oai(validator(max_length = "255"))]
    /// 接收组名称
    pub receive_group_name: Option<String>,
}

#[derive(Debug, poem_openapi::Object, Default)]
pub struct ReachTriggerInstanceConfigFilterReq {
    #[oai(flatten)]
    pub base_filter: RbumItemBasicFilterReq,
    #[oai(validator(max_length = "255"))]
    pub receive_group_code: Vec<String>,

    /// 关联的触达通道
    pub rel_reach_channel: Option<ReachChannelKind>,

    #[oai(validator(max_length = "512"))]
    /// 关联资源项id
    pub rel_item_id: Option<String>,

    #[oai(validator(max_length = "512"))]
    /// 关联的触发场景id
    pub rel_reach_trigger_scene_id: Option<String>,
}

#[derive(Debug, poem_openapi::Object, Serialize, sea_orm::FromQueryResult)]
pub struct ReachTriggerInstanceConfigSummaryResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    /// 关联的触发场景id
    pub rel_reach_trigger_scene_id: String,
    /// 关联的触达通道
    pub rel_reach_channel: ReachChannelKind,
    /// 关联资源项id
    pub rel_item_id: String,
    /// 接收组编码
    pub receive_group_code: String,
    /// 接收组名称
    pub receive_group_name: String,
}

#[derive(Debug, poem_openapi::Object, Serialize, sea_orm::FromQueryResult)]
pub struct ReachTriggerInstanceConfigDetailResp {
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
    /// 关联资源项id
    pub rel_item_id: String,
    /// 接收组编码
    pub receive_group_code: String,
    /// 接收组名称
    pub receive_group_name: String,
}
