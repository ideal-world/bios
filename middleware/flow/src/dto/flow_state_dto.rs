use std::str::FromStr;

use bios_basic::rbum::{
    dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher, RbumItemRelFilterReq},
    rbum_enumeration::RbumScopeLevelKind,
};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    db::sea_orm::{self, DbErr, QueryResult, TryGetError, TryGetable},
    derive_more::Display,
    serde_json::Value,
    web::poem_openapi,
};

use super::flow_var_dto::FlowVarSimpleInfo;

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowStateAddReq {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub id_prefix: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub icon: Option<String>,
    pub sys_state: FlowSysStateKind,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub info: Option<String>,
    pub vars: Option<Vec<FlowVarSimpleInfo>>,
    pub state_kind: Option<FlowStateKind>,
    pub kind_conf: Option<Value>,

    pub template: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_state_id: Option<String>,

    #[oai(validator(min_length = "2", max_length = "200"))]
    pub tag: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowStateModifyReq {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub icon: Option<String>,
    pub sys_state: Option<FlowSysStateKind>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub info: Option<String>,
    pub vars: Option<Vec<FlowVarSimpleInfo>>,
    pub state_kind: Option<FlowStateKind>,
    pub kind_conf: Option<Value>,

    pub template: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_state_id: Option<String>,

    #[oai(validator(min_length = "2", max_length = "200"))]
    pub tag: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowStateSummaryResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub sys_state: FlowSysStateKind,
    pub info: String,

    pub vars: Option<Value>,

    pub state_kind: FlowStateKind,
    pub kind_conf: Value,

    pub template: bool,
    pub rel_state_id: String,

    pub tag: String,

    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub disabled: bool,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowStateDetailResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub sys_state: FlowSysStateKind,
    pub info: String,

    pub vars: Option<Value>,

    pub state_kind: FlowStateKind,
    pub kind_conf: Value,

    pub template: bool,
    pub rel_state_id: String,

    pub tag: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum FlowSysStateKind {
    Start,
    Progress,
    Finish,
}

impl TryGetable for FlowSysStateKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        FlowSysStateKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum FlowStateKind {
    Simple,
    Form,
    Mail,
    Callback,
    Timer,
    Script,
}

impl TryGetable for FlowStateKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        FlowStateKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct FlowStateFilterReq {
    pub basic: RbumBasicFilterReq,
    pub sys_state: Option<FlowSysStateKind>,
    pub tag: Option<String>,
    pub state_kind: Option<FlowStateKind>,
    pub template: Option<bool>,
}

impl RbumItemFilterFetcher for FlowStateFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
    fn rel(&self) -> &Option<RbumItemRelFilterReq> {
        &None
    }
    fn rel2(&self) -> &Option<RbumItemRelFilterReq> {
        &None
    }
}
