use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::{
    basic::dto::TardisContext,
    chrono::{DateTime, Utc},
    db::sea_orm,
    serde_json::Value,
    web::poem_openapi,
};

use super::{flow_state_dto::FlowSysStateKind, flow_transition_dto::FlowTransitionDoubleCheckInfo, flow_var_dto::FlowVarInfo};

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstStartReq {
    pub rel_business_obj_id: String,
    pub tag: String,
    pub create_vars: Option<HashMap<String, Value>>,
    pub current_state_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstBindReq {
    pub tag: String,
    pub rel_business_objs: Vec<FlowInstBindRelObjReq>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstBindRelObjReq {
    pub rel_business_obj_id: String,
    pub current_state_name: String,
    pub own_paths: String,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstBindResp {
    pub rel_business_obj_id: String,
    pub current_state_name: String,
    pub inst_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstAbortReq {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstSummaryResp {
    pub id: String,
    pub rel_flow_model_id: String,
    pub rel_flow_model_name: String,
    pub rel_business_obj_id: String,

    pub current_state_id: String,
    pub current_assigned: Option<String>,

    pub create_ctx: FlowOperationContext,
    pub create_time: DateTime<Utc>,

    pub finish_ctx: Option<FlowOperationContext>,
    pub finish_time: Option<DateTime<Utc>>,
    pub finish_abort: bool,
    pub output_message: Option<String>,

    pub own_paths: String,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstDetailResp {
    pub id: String,
    pub rel_flow_model_id: String,
    pub rel_flow_model_name: String,
    pub rel_business_obj_id: String,

    pub current_state_id: String,
    pub current_assigned: Option<String>,

    pub current_state_name: Option<String>,
    pub current_vars: Option<HashMap<String, Value>>,

    pub create_vars: Option<HashMap<String, Value>>,
    pub create_ctx: FlowOperationContext,
    pub create_time: DateTime<Utc>,

    pub finish_ctx: Option<FlowOperationContext>,
    pub finish_time: Option<DateTime<Utc>>,
    pub finish_abort: Option<bool>,
    pub output_message: Option<String>,

    pub transitions: Option<Vec<FlowInstTransitionInfo>>,

    pub own_paths: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowInstTransitionInfo {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub op_ctx: FlowOperationContext,
    pub output_message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowOperationContext {
    pub own_paths: String,
    pub ak: String,
    pub owner: String,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
}

impl FlowOperationContext {
    pub fn from_ctx(ctx: &TardisContext) -> Self {
        FlowOperationContext {
            own_paths: ctx.own_paths.to_string(),
            ak: ctx.ak.to_string(),
            owner: ctx.owner.to_string(),
            roles: ctx.roles.clone(),
            groups: ctx.groups.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstFindNextTransitionsReq {
    pub vars: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstFindNextTransitionResp {
    pub next_flow_transition_id: String,
    pub next_flow_transition_name: String,
    pub next_flow_state_id: String,
    pub next_flow_state_name: String,

    pub vars_collect: Option<Vec<FlowVarInfo>>,

    pub double_check: Option<FlowTransitionDoubleCheckInfo>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstFindStateAndTransitionsReq {
    pub flow_inst_id: String,
    pub vars: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstFindStateAndTransitionsResp {
    pub flow_inst_id: String,
    pub current_flow_state_name: String,
    pub current_flow_state_kind: FlowSysStateKind,
    pub next_flow_transitions: Vec<FlowInstFindNextTransitionResp>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstTransferReq {
    pub flow_transition_id: String,
    pub message: Option<String>,
    pub vars: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstTransferResp {
    pub prev_flow_state_id: String,
    pub prev_flow_state_name: Option<String>,
    pub new_flow_state_id: String,
    pub new_flow_state_name: String,

    pub vars: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstModifyAssignedReq {
    pub current_assigned: String,
}
