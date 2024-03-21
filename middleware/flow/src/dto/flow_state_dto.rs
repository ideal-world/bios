use bios_basic::rbum::{
    dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher, RbumItemRelFilterReq},
    rbum_enumeration::RbumScopeLevelKind,
};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    db::sea_orm::{self, EnumIter},
    serde_json::Value,
    web::poem_openapi,
};

#[derive(Serialize, Deserialize, Default, Debug, poem_openapi::Object)]
pub struct FlowStateAddReq {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub id_prefix: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sys_state: FlowSysStateKind,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub info: Option<String>,
    pub state_kind: Option<FlowStateKind>,
    pub kind_conf: Option<Value>,

    pub template: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_state_id: Option<String>,

    #[oai(validator(min_length = "2", max_length = "200"))]
    pub tags: Option<Vec<String>>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowStateModifyReq {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sys_state: Option<FlowSysStateKind>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub info: Option<String>,
    pub state_kind: Option<FlowStateKind>,
    pub kind_conf: Option<Value>,

    pub template: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_state_id: Option<String>,

    #[oai(validator(min_length = "2", max_length = "200"))]
    pub tags: Option<Vec<String>>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowStateSummaryResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub sys_state: FlowSysStateKind,
    pub info: String,

    pub state_kind: FlowStateKind,
    pub kind_conf: Value,

    pub template: bool,
    pub rel_state_id: String,

    pub tags: String,

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
    pub color: String,
    pub sys_state: FlowSysStateKind,
    pub info: String,

    pub state_kind: FlowStateKind,
    pub kind_conf: Value,

    pub template: bool,
    pub rel_state_id: String,

    pub tags: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, EnumIter, sea_orm::DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(255))")]
pub enum FlowSysStateKind {
    #[default]
    #[sea_orm(string_value = "start")]
    Start,
    #[sea_orm(string_value = "progress")]
    Progress,
    #[sea_orm(string_value = "finish")]
    Finish,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, EnumIter, sea_orm::DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(255))")]
pub enum FlowStateKind {
    #[sea_orm(string_value = "simple")]
    Simple,
    #[sea_orm(string_value = "form")]
    Form,
    #[sea_orm(string_value = "mail")]
    Mail,
    #[sea_orm(string_value = "callback")]
    Callback,
    #[sea_orm(string_value = "timer")]
    Timer,
    #[sea_orm(string_value = "script")]
    Script,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct FlowStateFilterReq {
    pub basic: RbumBasicFilterReq,
    pub sys_state: Option<FlowSysStateKind>,
    pub tag: Option<String>,
    pub state_kind: Option<FlowStateKind>,
    pub template: Option<bool>,
    pub flow_model_ids: Option<Vec<String>>,
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

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowStateNameResp {
    pub key: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowStateCountGroupByStateReq {
    pub tag: String,
    pub inst_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowStateCountGroupByStateResp {
    pub state_name: String,
    pub count: String,
    pub inst_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowStateRelModelExt {
    pub sort: i64,
    pub show_btns: Option<Vec<String>>,
}
