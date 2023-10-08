use bios_basic::{basic_enumeration::BasicQueryOpKind, dto::BasicQueryCondInfo};
use serde::{Deserialize, Serialize};
use tardis::{basic::field::TrimString, db::sea_orm, serde_json::Value, web::poem_openapi, TardisFuns};

use super::flow_var_dto::FlowVarInfo;

#[derive(Serialize, Deserialize, Debug, Default, Clone, poem_openapi::Object)]
pub struct FlowTransitionAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_flow_state_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_flow_state_id: String,

    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: Option<TrimString>,

    pub transfer_by_auto: Option<bool>,
    pub transfer_by_timer: Option<String>,

    pub guard_by_creator: Option<bool>,
    pub guard_by_his_operators: Option<bool>,
    pub guard_by_assigned: Option<bool>,
    pub guard_by_spec_account_ids: Option<Vec<String>>,
    pub guard_by_spec_role_ids: Option<Vec<String>>,
    pub guard_by_spec_org_ids: Option<Vec<String>>,
    pub guard_by_other_conds: Option<Vec<Vec<BasicQueryCondInfo>>>,

    pub double_check: Option<FlowTransitionDoubleCheckInfo>,
    pub vars_collect: Option<Vec<FlowVarInfo>>,

    pub action_by_pre_callback: Option<String>,
    pub action_by_post_callback: Option<String>,
    pub action_by_post_changes: Option<Vec<FlowTransitionActionChangeInfo>>,
    pub action_by_front_changes: Option<Vec<FlowTransitionFrontActionInfo>>,

    pub sort: Option<i64>,
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
    pub transfer_by_timer: Option<String>,

    pub guard_by_creator: Option<bool>,
    pub guard_by_his_operators: Option<bool>,
    pub guard_by_assigned: Option<bool>,
    pub guard_by_spec_account_ids: Option<Vec<String>>,
    pub guard_by_spec_role_ids: Option<Vec<String>>,
    pub guard_by_spec_org_ids: Option<Vec<String>>,
    pub guard_by_other_conds: Option<Vec<Vec<BasicQueryCondInfo>>>,

    pub vars_collect: Option<Vec<FlowVarInfo>>,
    pub double_check: Option<FlowTransitionDoubleCheckInfo>,

    pub action_by_pre_callback: Option<String>,
    pub action_by_post_callback: Option<String>,
    pub action_by_post_changes: Option<Vec<FlowTransitionActionChangeInfo>>,
    pub action_by_front_changes: Option<Vec<FlowTransitionFrontActionInfo>>,

    pub sort: Option<i64>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowTransitionDetailResp {
    pub id: String,
    pub name: String,

    pub from_flow_state_id: String,
    pub from_flow_state_name: String,
    pub from_flow_state_color: String,
    pub to_flow_state_id: String,
    pub to_flow_state_name: String,
    pub to_flow_state_color: String,

    pub transfer_by_auto: bool,
    pub transfer_by_timer: String,

    pub guard_by_creator: bool,
    pub guard_by_his_operators: bool,
    pub guard_by_assigned: bool,
    pub guard_by_spec_account_ids: Vec<String>,
    pub guard_by_spec_role_ids: Vec<String>,
    pub guard_by_spec_org_ids: Vec<String>,
    // TODO
    pub guard_by_other_conds: Value,

    pub vars_collect: Value,
    pub double_check: Value,

    pub action_by_pre_callback: String,
    pub action_by_post_callback: String,
    pub action_by_post_changes: Value,
    pub action_by_front_changes: Value,

    pub rel_flow_model_id: String,
    pub sort: i64,
}

impl FlowTransitionDetailResp {
    pub fn guard_by_other_conds(&self) -> Option<Vec<Vec<BasicQueryCondInfo>>> {
        if self.guard_by_other_conds.is_array() && !&self.guard_by_other_conds.as_array().unwrap().is_empty() {
            Some(TardisFuns::json.json_to_obj(self.guard_by_other_conds.clone()).unwrap_or_default())
        } else {
            None
        }
    }

    pub fn vars_collect(&self) -> Option<Vec<FlowVarInfo>> {
        if self.vars_collect.is_array() && !&self.vars_collect.as_array().unwrap().is_empty() {
            Some(TardisFuns::json.json_to_obj(self.vars_collect.clone()).unwrap_or_default())
        } else {
            None
        }
    }

    pub fn action_by_post_changes(&self) -> Vec<FlowTransitionActionChangeInfo> {
        if self.action_by_post_changes.is_array() && !&self.action_by_post_changes.as_array().unwrap().is_empty() {
            TardisFuns::json.json_to_obj(self.action_by_post_changes.clone()).unwrap_or_default()
        } else {
            vec![]
        }
    }

    pub fn action_by_front_changes(&self) -> Vec<FlowTransitionFrontActionInfo> {
        if self.action_by_front_changes.is_array() && !&self.action_by_front_changes.as_array().unwrap().is_empty() {
            TardisFuns::json.json_to_obj(self.action_by_front_changes.clone()).unwrap_or_default()
        } else {
            vec![]
        }
    }

    pub fn double_check(&self) -> Option<FlowTransitionDoubleCheckInfo> {
        if self.double_check.is_object() {
            Some(TardisFuns::json.json_to_obj(self.double_check.clone()).unwrap_or_default())
        } else {
            None
        }
    }
}

impl From<FlowTransitionDetailResp> for FlowTransitionAddReq {
    fn from(value: FlowTransitionDetailResp) -> Self {
        let guard_by_other_conds = value.guard_by_other_conds();
        let vars_collect = value.vars_collect();
        let action_by_post_changes = value.action_by_post_changes();
        let action_by_front_changes = value.action_by_front_changes();
        let double_check = value.double_check();
        FlowTransitionAddReq {
            from_flow_state_id: value.from_flow_state_id,
            to_flow_state_id: value.to_flow_state_id,
            name: Some(value.name.into()),
            transfer_by_auto: Some(value.transfer_by_auto),
            transfer_by_timer: Some(value.transfer_by_timer),
            guard_by_creator: Some(value.guard_by_creator),
            guard_by_his_operators: Some(value.guard_by_his_operators),
            guard_by_assigned: Some(value.guard_by_assigned),
            guard_by_spec_account_ids: Some(value.guard_by_spec_account_ids),
            guard_by_spec_role_ids: Some(value.guard_by_spec_role_ids),
            guard_by_spec_org_ids: Some(value.guard_by_spec_org_ids),
            guard_by_other_conds,
            vars_collect,
            action_by_pre_callback: Some(value.action_by_pre_callback),
            action_by_post_callback: Some(value.action_by_post_callback),
            action_by_post_changes: Some(action_by_post_changes),
            action_by_front_changes: Some(action_by_front_changes),
            double_check,
            sort: Some(value.sort),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default, poem_openapi::Object)]
pub struct FlowTransitionDoubleCheckInfo {
    pub is_open: bool,
    pub content: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowTransitionSortStatesReq {
    pub sort_states: Vec<FlowTransitionSortStateInfoReq>,
}

#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowTransitionSortStateInfoReq {
    pub id: String,
    pub sort: i64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object)]
pub struct FlowTransitionActionChangeInfo {
    pub kind: FlowTransitionActionChangeKind,
    pub describe: String,
    pub obj_tag: Option<String>,
    pub obj_current_state_id: Option<Vec<String>>,
    pub change_condition: Option<StateChangeCondition>,
    pub changed_state_id: String,
    pub current: bool,
    pub var_name: String,
    pub changed_val: Option<Value>,
    pub changed_current_time: Option<bool>,
}

impl From<FlowTransitionActionChangeInfo> for FlowTransitionActionChangeAgg {
    fn from(value: FlowTransitionActionChangeInfo) -> Self {
        match value.kind {
            FlowTransitionActionChangeKind::State => FlowTransitionActionChangeAgg {
                kind: value.kind,
                var_change_info: None,
                state_change_info: Some(FlowTransitionActionByStateChangeInfo {
                    obj_tag: value.obj_tag.unwrap(),
                    describe: value.describe,
                    obj_current_state_id: value.obj_current_state_id,
                    change_condition: value.change_condition,
                    changed_state_id: value.changed_state_id,
                }),
            },
            FlowTransitionActionChangeKind::Var => FlowTransitionActionChangeAgg {
                kind: value.kind,
                var_change_info: Some(FlowTransitionActionByVarChangeInfo {
                    current: value.current,
                    describe: value.describe,
                    obj_tag: value.obj_tag,
                    var_name: value.var_name,
                    changed_val: value.changed_val,
                    changed_current_time: value.changed_current_time,
                }),
                state_change_info: None,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object)]
pub struct FlowTransitionActionChangeAgg {
    pub kind: FlowTransitionActionChangeKind,
    pub var_change_info: Option<FlowTransitionActionByVarChangeInfo>,
    pub state_change_info: Option<FlowTransitionActionByStateChangeInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumIter, sea_orm::DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(255))")]
pub enum FlowTransitionActionChangeKind {
    #[sea_orm(string_value = "var")]
    Var,
    #[sea_orm(string_value = "state")]
    State,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowTransitionActionByVarChangeInfo {
    pub current: bool,
    pub describe: String,
    pub obj_tag: Option<String>,
    pub var_name: String,
    pub changed_val: Option<Value>,
    pub changed_current_time: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowTransitionActionByStateChangeInfo {
    pub obj_tag: String,
    pub describe: String,
    pub obj_current_state_id: Option<Vec<String>>,
    pub change_condition: Option<StateChangeCondition>,
    pub changed_state_id: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct StateChangeCondition {
    pub current: bool,
    pub conditions: Vec<StateChangeConditionItem>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct StateChangeConditionItem {
    pub obj_tag: Option<String>,
    pub state_id: Vec<String>,
    pub op: StateChangeConditionOp,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Enum)]
pub enum StateChangeConditionOp {
    And,
    Or,
}

#[derive(Default)]
pub struct FlowTransitionInitInfo {
    pub from_flow_state_name: String,
    pub to_flow_state_name: String,
    pub name: String,
    pub transfer_by_auto: Option<bool>,
    pub transfer_by_timer: Option<String>,

    pub guard_by_creator: Option<bool>,
    pub guard_by_his_operators: Option<bool>,
    pub guard_by_assigned: Option<bool>,
    pub guard_by_spec_account_ids: Option<Vec<String>>,
    pub guard_by_spec_role_ids: Option<Vec<String>>,
    pub guard_by_spec_org_ids: Option<Vec<String>>,
    pub guard_by_other_conds: Option<Vec<Vec<BasicQueryCondInfo>>>,

    pub vars_collect: Option<Vec<FlowVarInfo>>,
    pub double_check: Option<FlowTransitionDoubleCheckInfo>,

    pub action_by_pre_callback: Option<String>,
    pub action_by_post_callback: Option<String>,
    pub action_by_post_changes: Vec<FlowTransitionActionChangeInfo>,
    pub action_by_front_changes: Vec<FlowTransitionFrontActionInfo>,

    pub sort: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object)]
pub struct FlowTransitionFrontActionInfo {
    pub op: BasicQueryOpKind,
    pub op_label: String,
    pub left_value: String,
    pub left_label: String,
    pub right_value: FlowTransitionFrontActionRightValue,
    pub select_field: Option<String>,
    pub select_field_label: Option<String>,
    pub change_content: Option<String>,
    pub change_content_label: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Enum)]
pub enum FlowTransitionFrontActionRightValue {
    #[oai(rename = "select_field")]
    SelectField,
    #[oai(rename = "change_content")]
    ChangeContent,
    #[oai(rename = "real_time")]
    RealTime,
}
