use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::{
    basic::dto::TardisContext,
    chrono::{DateTime, Utc},
    db::sea_orm,
    serde_json::Value,
    web::poem_openapi,
};

use super::{
    flow_state_dto::{FlowStateRelModelExt, FlowSysStateKind},
    flow_transition_dto::FlowTransitionDoubleCheckInfo,
    flow_var_dto::FlowVarInfo,
};

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstStartReq {
    /// 关联业务ID
    pub rel_business_obj_id: String,
    pub tag: String,
    /// 创建时的参数列表
    pub create_vars: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstBindReq {
    /// 关联业务ID
    pub rel_business_obj_id: String,
    pub tag: String,
    pub create_vars: Option<HashMap<String, Value>>,
    /// 创建时的状态名
    pub current_state_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstBatchBindReq {
    pub tag: String,
    /// 关联业务ID
    pub rel_business_objs: Vec<FlowInstBindRelObjReq>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstBindRelObjReq {
    /// 关联业务ID
    pub rel_business_obj_id: Option<String>,
    /// 当前状态名
    pub current_state_name: Option<String>,
    pub own_paths: Option<String>,
    pub owner: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstBatchBindResp {
    /// 关联业务ID
    pub rel_business_obj_id: String,
    /// 当前状态名
    pub current_state_name: String,
    /// 实例ID
    pub inst_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstAbortReq {
    pub message: String,
}

/// 工作流实例的概要信息
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstSummaryResp {
    pub id: String,
    /// Associated [flow_model](super::flow_model_dto::FlowModelDetailResp) id
    ///
    /// 关联的[工作流模板](super::flow_model_dto::FlowModelDetailResp) id
    pub rel_flow_model_id: String,
    /// Associated [flow_model](super::flow_model_dto::FlowModelDetailResp) name
    ///
    /// 关联的[工作流模板](super::flow_model_dto::FlowModelDetailResp) 名称
    pub rel_flow_model_name: String,
    /// 关联业务ID
    pub rel_business_obj_id: String,
    /// 当前状态ID
    pub current_state_id: String,
    /// 创建上下文信息
    pub create_ctx: FlowOperationContext,
    /// 创建时间
    pub create_time: DateTime<Utc>,
    /// 结束上下文信息
    pub finish_ctx: Option<FlowOperationContext>,
    /// 结束时间
    pub finish_time: Option<DateTime<Utc>>,
    /// 是否异常终止
    pub finish_abort: bool,
    /// 输出信息
    pub output_message: Option<String>,

    pub own_paths: String,
}

/// 工作流详细信息
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstDetailResp {
    pub id: String,
    /// Associated [flow_model](super::flow_model_dto::FlowModelDetailResp) id
    ///
    /// 关联的[工作流模板](super::flow_model_dto::FlowModelDetailResp) id
    pub rel_flow_model_id: String,
    /// Associated [flow_model](super::flow_model_dto::FlowModelDetailResp) name
    ///
    /// 关联的[工作流模板](super::flow_model_dto::FlowModelDetailResp) 名称
    pub rel_flow_model_name: String,
    /// 关联业务ID
    pub rel_business_obj_id: String,
    /// 当前状态ID
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) id
    pub current_state_id: String,
    /// 当前状态名称
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) name
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) name
    pub current_state_name: Option<String>,
    /// 当前状态颜色信息
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) color
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) color
    pub current_state_color: Option<String>,
    /// 当前状态类型
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) name
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) 名称
    pub current_state_kind: Option<FlowSysStateKind>,
    /// 当前状态类型
    /// Associated [flow_state](super::flow_state_dto::FlowStateRelModelExt)
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateRelModelExt)
    pub current_state_ext: Option<FlowStateRelModelExt>,
    /// 当前参数列表
    pub current_vars: Option<HashMap<String, Value>>,
    /// 创建时的参数列表
    pub create_vars: Option<HashMap<String, Value>>,
    /// 创建上下文
    pub create_ctx: FlowOperationContext,
    /// 创建时间
    pub create_time: DateTime<Utc>,

    /// 结束上下文
    pub finish_ctx: Option<FlowOperationContext>,
    /// 结束时间
    pub finish_time: Option<DateTime<Utc>>,
    /// 是否异常终止
    pub finish_abort: Option<bool>,
    /// 输出信息
    pub output_message: Option<String>,
    /// 动作列表
    pub transitions: Option<Vec<FlowInstTransitionInfo>>,

    pub own_paths: String,
}

/// 实例的动作信息
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowInstTransitionInfo {
    pub id: String,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 操作上下文
    pub op_ctx: FlowOperationContext,
    /// 输出信息
    pub output_message: Option<String>,
}

/// 操作上下文信息
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

/// 获取实例下一个动作列表请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstFindNextTransitionsReq {
    pub vars: Option<HashMap<String, Value>>,
}

/// 获取实例下一个动作列表请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, Clone)]
pub struct FlowInstFindNextTransitionResp {
    pub next_flow_transition_id: String,
    pub next_flow_transition_name: String,
    pub next_flow_state_id: String,
    pub next_flow_state_name: String,
    pub next_flow_state_color: String,

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
    pub current_flow_state_color: String,
    pub current_flow_state_ext: FlowStateRelModelExt,
    pub finish_time: Option<DateTime<Utc>>,
    pub next_flow_transitions: Vec<FlowInstFindNextTransitionResp>,
}

#[derive(Serialize, Deserialize, Clone, Debug, poem_openapi::Object)]
pub struct FlowInstTransferReq {
    pub flow_transition_id: String,
    pub message: Option<String>,
    pub vars: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstTransferResp {
    pub prev_flow_state_id: String,
    pub prev_flow_state_name: String,
    pub prev_flow_state_color: String,
    pub new_flow_state_id: String,
    pub new_flow_state_name: String,
    pub new_flow_state_color: String,
    pub new_flow_state_ext: FlowStateRelModelExt,
    pub finish_time: Option<DateTime<Utc>>,

    pub vars: Option<HashMap<String, Value>>,
    pub next_flow_transitions: Vec<FlowInstFindNextTransitionResp>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstModifyAssignedReq {
    pub current_assigned: String,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstModifyCurrentVarsReq {
    pub vars: HashMap<String, Value>,
}
