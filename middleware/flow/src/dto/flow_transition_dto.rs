use bios_basic::dto::BasicQueryCondInfo;
use serde::{Deserialize, Serialize};
use tardis::{basic::field::TrimString, db::sea_orm, serde_json::Value, web::poem_openapi, TardisFuns};

use super::flow_var_dto::FlowVarInfo;

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowTransitionAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_flow_state_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_flow_state_id: String,

    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: Option<TrimString>,

    pub transfer_by_auto: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub transfer_by_timer: Option<String>,

    pub guard_by_creator: Option<bool>,
    pub guard_by_his_operators: Option<bool>,
    pub guard_by_spec_account_ids: Option<Vec<String>>,
    pub guard_by_spec_role_ids: Option<Vec<String>>,
    pub guard_by_other_conds: Option<Vec<Vec<BasicQueryCondInfo>>>,

    pub vars_collect: Option<Vec<FlowVarInfo>>,

    pub action_by_pre_callback: Option<String>,
    pub action_by_post_callback: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowTransitionModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub id: TrimString,
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_flow_state_id: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_flow_state_id: Option<String>,

    pub transfer_by_auto: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub transfer_by_timer: Option<String>,

    pub guard_by_creator: Option<bool>,
    pub guard_by_his_operators: Option<bool>,
    pub guard_by_spec_account_ids: Option<Vec<String>>,
    pub guard_by_spec_role_ids: Option<Vec<String>>,
    pub guard_by_other_conds: Option<Vec<Vec<BasicQueryCondInfo>>>,

    pub vars_collect: Option<Vec<FlowVarInfo>>,

    pub action_by_pre_callback: Option<String>,
    pub action_by_post_callback: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowTransitionDetailResp {
    pub id: String,
    pub name: String,

    pub from_flow_state_id: String,
    pub from_flow_state_name: String,
    pub to_flow_state_id: String,
    pub to_flow_state_name: String,

    pub transfer_by_auto: bool,
    pub transfer_by_timer: String,

    pub guard_by_creator: bool,
    pub guard_by_his_operators: bool,
    pub guard_by_spec_account_ids: Vec<String>,
    pub guard_by_spec_role_ids: Vec<String>,
    // TODO
    pub guard_by_other_conds: Value,

    pub vars_collect: Value,

    pub action_by_pre_callback: String,
    pub action_by_post_callback: String,
}

impl FlowTransitionDetailResp {
    pub fn guard_by_other_conds(&self) -> Option<Vec<Vec<BasicQueryCondInfo>>> {
        if self.guard_by_other_conds.is_array() && !&self.guard_by_other_conds.as_array().unwrap().is_empty() {
            Some(TardisFuns::json.json_to_obj(self.guard_by_other_conds.clone()).unwrap())
        } else {
            None
        }
    }

    pub fn vars_collect(&self) -> Option<Vec<FlowVarInfo>> {
        if self.vars_collect.is_array() && !&self.vars_collect.as_array().unwrap().is_empty() {
            Some(TardisFuns::json.json_to_obj(self.vars_collect.clone()).unwrap())
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowTransitionActionByVarChangeInfo {
    pub current: bool,
    pub obj_tag: Option<String>,
    pub var_name: String,
    pub changed_val: Value,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowTransitionActionByStateChangeInfo {
    pub obj_tag: String,
    pub obj_state_ids: Vec<String>,
    pub changed_state_id: String,
}
