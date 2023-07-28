use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    db::sea_orm::{self},
    web::poem_openapi,
};


use crate::dto::*;
/// 添加用户触达签名请求
#[derive(Debug, poem_openapi::Object, Deserialize)]
pub struct ReachMsgSignatureAddReq {
    /// 名称
    #[oai(validator(max_length = "255"))]
    pub name: String,
    /// 说明
    #[oai(validator(max_length = "2000"))]
    pub note: String,
    /// 内容
    #[oai(validator(max_length = "2000"))]
    pub content: String,
    /// 来源
    #[oai(validator(max_length = "255"))]
    pub source: String,
    pub rel_reach_channel: ReachChannelKind,
}
/// 修改用户触达签名请求
#[derive(Debug, poem_openapi::Object, Deserialize)]
pub struct ReachMsgSignatureModifyReq {
    /// 名称
    #[oai(validator(max_length = "255"))]
    pub name: Option<String>,
    /// 说明
    #[oai(validator(max_length = "2000"))]
    pub note: Option<String>,
    /// 内容
    #[oai(validator(max_length = "2000"))]
    pub content: Option<String>,
    /// 来源
    #[oai(validator(max_length = "255"))]
    pub source: Option<String>,
    pub rel_reach_channel: Option<ReachChannelKind>,
}

/// 用户触达签名过滤请求
#[derive(Debug, poem_openapi::Object, Default)]
pub struct ReachMsgSignatureFilterReq {
    #[oai(flatten)]
    pub base_filter: RbumBasicFilterReq,
    /// 名称
    #[oai(validator(min_length = "1", max_length = "255"))]
    pub name: String,
    /// 关联的触达通道
    pub rel_reach_channel: Option<ReachChannelKind>,
}

#[derive(Debug, poem_openapi::Object, sea_orm::FromQueryResult, Serialize)]
pub struct ReachMsgSignatureSummaryResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub name: String,
    pub note: String,
    pub content: String,
    pub source: String,
    pub rel_reach_channel: ReachChannelKind,
}

#[derive(Debug, poem_openapi::Object, sea_orm::FromQueryResult, Serialize)]
pub struct ReachMsgSignatureDetailResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub name: String,
    pub note: String,
    pub content: String,
    pub source: String,
    pub rel_reach_channel: ReachChannelKind,
}
