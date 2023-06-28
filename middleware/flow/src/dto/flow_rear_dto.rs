use serde::{Deserialize, Serialize};
use tardis::db::sea_orm::strum::Display;
use tardis::web::poem_openapi;

use super::flow_model_dto::FlowTagKind;

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowRearActionInfo {
    pub action_kind: FlowRearActionKind,
    pub state_modify_conf: Option<FlowRearStateModifyConf>,
    pub field_modify_conf: Option<FlowRearFieldModifyConf>,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum FlowRearActionKind {
    StateModify,
    FieldModify,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowRearStateModifyConf {
    pub rel_tag: FlowTagKind,
    pub conds: Vec<FlowRearStateCond>,
    pub to_state_id: String,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowRearStateCond {
    pub cond_kind: FlowRearStateCondKind,
    pub rel_kind: Option<FlowTagKind>,
    pub current_state_id: String,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum FlowRearStateCondKind {
    Current,
    Rel,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowRearFieldModifyConf {
    pub modify_kind: FlowRearFieldCondKind,
    pub rel_kind: Option<FlowTagKind>,
    pub field_kind: String,
    pub to_data_id: String,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum FlowRearFieldCondKind {
    Current,
    Rel,
}
