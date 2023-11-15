use serde::{Deserialize, Serialize};
use serde_json::Value;
use tardis::web::poem_openapi::{
    self,
    types::{ParseFromJSON, ToJSON},
};

use super::flow_state_dto::FlowSysStateKind;

#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowExternalReq {
    pub kind: FlowExternalKind,
    pub curr_tag: String,
    pub curr_bus_obj_id: String,
    pub inst_id: String,
    pub target_state: Option<String>,
    pub target_sys_state: Option<FlowSysStateKind>,
    pub original_state: Option<String>,
    pub original_sys_state: Option<FlowSysStateKind>,
    pub owner_paths: String,
    pub obj_ids: Vec<String>,
    pub params: Vec<FlowExternalParams>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum FlowExternalKind {
    #[default]
    FetchRelObj,
    ModifyField,
    NotifyChanges,
    QueryField,
}

#[derive(Debug, Deserialize, Serialize, poem_openapi::Object, Clone)]
pub struct FlowExternalParams {
    pub rel_tag: Option<String>,
    pub rel_kind: Option<String>,
    pub var_id: Option<String>,
    pub var_name: Option<String>,
    pub value: Option<Value>,
}

#[derive(Default, Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalResp<T>
where
    T: ParseFromJSON + ToJSON + Serialize + Send + Sync,
{
    pub code: String,
    pub message: String,
    pub body: Option<T>,
}

#[derive(Default, Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalFetchRelObjResp {
    pub curr_tag: String,
    pub curr_bus_obj_id: String,
    pub rel_bus_objs: Vec<RelBusObjResp>,
}

#[derive(Default, Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct RelBusObjResp {
    pub rel_tag: String,
    pub rel_bus_obj_ids: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalModifyFieldResp {}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalNotifyChangesResp {}

#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowExternalQueryFieldResp {
    pub objs: Vec<Value>,
}
