use bios_basic::rbum::{
    dto::{rbum_filer_dto::RbumItemBasicFilterReq, rbum_item_dto::RbumItemAddReq},
    rbum_enumeration::RbumScopeLevelKind, serv::rbum_crud_serv::RbumCrudOperation,
};
use tardis::{basic::{field::TrimString, dto::TardisContext, result::TardisResult}, TardisFunsInst};

use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    db::sea_orm,
    web::poem_openapi,
};

use crate::serv::ReachTriggerSceneService;

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


pub struct ReachTriggerSceneTree {
    pub name: String,
    pub code: String,
    pub children: Vec<Self>,
}

impl ReachTriggerSceneTree {
    pub fn new(name: impl Into<String>, code: impl Into<String>, children: impl IntoIterator<Item = Self>) -> Self {
        ReachTriggerSceneTree {
            name: name.into(),
            code: code.into(),
            children: children.into_iter().collect(),
        }
    }
    pub(crate) async fn add(&self, pid: Option<&str>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mut stack = vec![(pid.map(String::from), self)];
        while let Some((pid, scene)) = stack.pop() {
            let mut add_req = ReachTriggerSceneAddReq::new_with_name_code(&self.name, &self.code);
            if let Some(pid) = pid {
                add_req = add_req.pid(&pid)
            }
            let id = ReachTriggerSceneService::add_rbum(&mut add_req, funs, ctx).await?;
            stack.extend(scene.children.iter().map(|child|(Some(id.clone()), child)));
        }
        Ok(())
    }
}