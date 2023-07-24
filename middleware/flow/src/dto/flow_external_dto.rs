use serde::{Deserialize, Serialize};
use serde_json::Value;
use tardis::web::poem_openapi;

use super::flow_transition_dto::StateChangeCondition;

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalReq {
    pub kind: FlowExternalKind,
    pub curr_tag: String,
    pub curr_bus_obj_id: String,
    pub params: FlowExternalParams,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum FlowExternalKind {
    FetchRelObj,
    ModifyField,
    NotifyChanges,
}

#[derive(Debug, Deserialize, Serialize, poem_openapi::Union)]
pub enum FlowExternalParams {
    FetchRelObj(FlowExternalFetchRelObjReq),
    ModifyField(FlowExternalModifyFieldReq),
    NotifyChanges(FlowExternalNotifyChangesReq),
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalFetchRelObjReq {
    pub obj_tag: String,
    pub obj_current_state_id: Option<String>,
    pub change_condition: Option<StateChangeCondition>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalFetchRelObjResp {
    pub rel_bus_obj_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalModifyFieldReq {
    pub current: bool,
    pub rel_tag: String,
    pub var_name: String,
    pub value: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalModifyFieldResp {}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalNotifyChangesReq {
    pub rel_tag: String,
    pub rel_bus_obj_ids: Vec<String>,
    pub state_id: Option<String>,
    pub var_name: Option<String>,
    pub value: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalNotifyChangesResp {}
