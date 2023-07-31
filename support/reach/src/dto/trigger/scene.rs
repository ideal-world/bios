use bios_basic::rbum::{
    dto::{rbum_filer_dto::RbumItemBasicFilterReq, rbum_item_dto::RbumItemAddReq},
    rbum_enumeration::RbumScopeLevelKind,
};
use tardis::basic::field::TrimString;

use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    db::sea_orm,
    web::poem_openapi,
};

/// 添加用户触达触发场景请求
#[derive(Debug, poem_openapi::Object)]
pub struct ReachTriggerSceneAddReq {
    #[oai(flatten)]
    pub rbum_add_req: RbumItemAddReq,
    #[oai(validator(max_length = "2000"))]
    /// 父场景ID
    pub pid: Option<String>,
}

impl ReachTriggerSceneAddReq {
    pub fn new_with_name_code(name: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            rbum_add_req: RbumItemAddReq {
                id: None,
                code: Some(TrimString(code.into())),
                name: TrimString(name.into()),
                rel_rbum_kind_id: Default::default(),
                rel_rbum_domain_id: Default::default(),
                scope_level: Some(RbumScopeLevelKind::Private),
                disabled: Some(false),
            },
            pid: Default::default(),
        }
    }
    pub fn pid(mut self, pid: impl Into<String>) -> Self {
        self.pid = Some(pid.into());
        self
    }
}
#[derive(Debug, poem_openapi::Object)]
pub struct ReachTriggerSceneModifyReq {
    #[oai(validator(max_length = "255"))]
    /// 编码
    pub code: String,
    #[oai(validator(max_length = "255"))]
    /// 名称
    pub name: String,
}

/// 用户触达触发场景过滤请求
#[derive(Debug, poem_openapi::Object, Default)]
pub struct ReachTriggerSceneFilterReq {
    #[oai(flatten)]
    pub base_filter: RbumItemBasicFilterReq,
    #[oai(validator(max_length = "255"))]
    /// 编码
    pub code: Option<String>,
    #[oai(validator(max_length = "255"))]
    /// 名称
    pub name: Option<String>,
}

#[derive(Debug, poem_openapi::Object, Serialize, Deserialize, sea_orm::FromQueryResult)]
pub struct ReachTriggerSceneSummaryResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    /// 编码
    pub code: String,
    /// 名称
    pub name: String,
    /// 父场景ID
    pub pid: String,
}

#[derive(Debug, poem_openapi::Object, Serialize, Deserialize, sea_orm::FromQueryResult)]
pub struct ReachTriggerSceneDetailResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    /// 编码
    pub code: String,
    /// 名称
    pub name: String,
    /// 父场景ID
    pub pid: String,
}
