use std::collections::HashMap;

use bios_basic::rbum::{
    dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher, RbumItemRelFilterReq},
    rbum_enumeration::RbumScopeLevelKind,
};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    db::sea_orm,
    serde_json::Value,
    web::poem_openapi,
    TardisFuns,
};

use super::flow_transition_dto::{FlowTransitionAddReq, FlowTransitionDetailResp, FlowTransitionModifyReq};

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowModelAddReq {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub icon: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub info: Option<String>,

    pub init_state_id: String,

    pub transitions: Option<Vec<FlowTransitionAddReq>>,

    pub template: bool,
    pub rel_model_id: Option<String>,

    pub tag: Option<FlowTagKind>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowModelModifyReq {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub icon: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub info: Option<String>,

    pub init_state_id: Option<String>,

    pub template: Option<bool>,

    pub add_transitions: Option<Vec<FlowTransitionAddReq>>,
    pub modify_transitions: Option<Vec<FlowTransitionModifyReq>>,
    pub delete_transitions: Option<Vec<String>>,

    pub tag: Option<FlowTagKind>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowModelSummaryResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub info: String,

    pub init_state_id: String,

    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub tag: String,

    pub disabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowModelDetailResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub info: String,

    pub init_state_id: String,

    // TODO
    pub transitions: Option<Value>,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub tag: FlowTagKind,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}

impl FlowModelDetailResp {
    pub fn transitions(&self) -> Vec<FlowTransitionDetailResp> {
        match &self.transitions {
            Some(transitions) => TardisFuns::json.json_to_obj(transitions.clone()).unwrap(),
            None => vec![],
        }
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct FlowModelFilterReq {
    pub basic: RbumBasicFilterReq,
    pub tag: Option<FlowTagKind>,
}

impl RbumItemFilterFetcher for FlowModelFilterReq {
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

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct FlowModelAggResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub info: String,

    pub init_state_id: String,

    pub states: HashMap<String, FlowStateAggResp>,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub tag: FlowTagKind,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct FlowStateAggResp {
    pub id: String,
    pub name: String,
    pub is_init: bool,
    pub transitions: Vec<FlowTransitionDetailResp>,
}

#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowModelAddStateReq {
    pub state_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumIter, sea_orm::DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(255))")]
pub enum FlowTagKind {
    #[sea_orm(string_value = "ticket_states")]
    Ticket,
    #[sea_orm(string_value = "proj_states")]
    Project,
    #[sea_orm(string_value = "milestone_states")]
    Milestone,
    #[sea_orm(string_value = "iter_states")]
    Iter,
}
