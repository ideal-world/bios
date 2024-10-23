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

    pub tag: String,
}

/// 工作流详细信息
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstDetailResp {
    pub id: String,
    /// Associated [flow_model](super::flow_model_dto::FlowModelDetailResp) id
    ///
    /// 关联的[工作流模板](super::flow_model_dto::FlowModelDetailResp) id
    pub rel_flow_version_id: String,
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
#[derive(Serialize, Deserialize, Clone, PartialEq, Default, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
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
    /// 参数列表
    pub vars: Option<HashMap<String, Value>>,
}

/// 获取实例下一个动作列表请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, Clone)]
pub struct FlowInstFindNextTransitionResp {
    /// Associated [flow_transition](super::flow_transition_dto::FlowTransitionDetailResp) id
    ///
    /// 关联的[工作流动态](super::flow_transition_dto::FlowTransitionDetailResp) id
    pub next_flow_transition_id: String,
    /// Associated [flow_transition](super::flow_transition_dto::FlowTransitionDetailResp) name
    ///
    /// 关联的[工作流动态](super::flow_transition_dto::FlowTransitionDetailResp) name
    pub next_flow_transition_name: String,
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) id
    pub next_flow_state_id: String,
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) name
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) name
    pub next_flow_state_name: String,
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) color
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) color
    pub next_flow_state_color: String,
    /// 参数列表
    pub vars_collect: Option<Vec<FlowVarInfo>>,
    /// Associated [二次确认](FlowTransitionDoubleCheckInfo)
    ///
    /// 关联的[二次确认](FlowTransitionDoubleCheckInfo)
    pub double_check: Option<FlowTransitionDoubleCheckInfo>,
}

/// 获取实例状态及流转信息的请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstFindStateAndTransitionsReq {
    /// 实例ID
    pub flow_inst_id: String,
    /// 参数列表
    pub vars: Option<HashMap<String, Value>>,
}

/// 实例状态及流转信息
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstFindStateAndTransitionsResp {
    /// 实例ID
    pub flow_inst_id: String,
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) name
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) name
    pub current_flow_state_name: String,
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) sys_state
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) sys_state
    pub current_flow_state_kind: FlowSysStateKind,
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) color
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) color
    pub current_flow_state_color: String,
    /// Associated [flow_state_ext](FlowStateRelModelExt)
    ///
    /// 关联的[工作流状态扩展](FlowStateRelModelExt)
    pub current_flow_state_ext: FlowStateRelModelExt,
    /// 结束时间
    pub finish_time: Option<DateTime<Utc>>,
    /// 流转信息
    pub next_flow_transitions: Vec<FlowInstFindNextTransitionResp>,
}

/// 流转请求
#[derive(Serialize, Deserialize, Clone, Debug, poem_openapi::Object)]
pub struct FlowInstTransferReq {
    /// 工作流实例ID
    pub flow_transition_id: String,
    /// 消息内容
    pub message: Option<String>,
    /// 参数列表
    pub vars: Option<HashMap<String, Value>>,
}

/// 流转响应
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstTransferResp {
    /// Associated [Pre-modification status](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[修改前状态](super::flow_state_dto::FlowStateDetailResp) id
    pub prev_flow_state_id: String,
    /// Associated [Pre-modification status](super::flow_state_dto::FlowStateDetailResp) name
    ///
    /// 关联的[修改前状态](super::flow_state_dto::FlowStateDetailResp) name
    pub prev_flow_state_name: String,
    /// Associated [Pre-modification status](super::flow_state_dto::FlowStateDetailResp) color
    ///
    /// 关联的[修改前状态](super::flow_state_dto::FlowStateDetailResp) color
    pub prev_flow_state_color: String,
    /// Associated [modified state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[修改后状态](super::flow_state_dto::FlowStateDetailResp) id
    pub new_flow_state_id: String,
    /// Associated [modified state](super::flow_state_dto::FlowStateDetailResp) name
    ///
    /// 关联的[修改后状态](super::flow_state_dto::FlowStateDetailResp) name
    pub new_flow_state_name: String,
    /// Associated [modified state](super::flow_state_dto::FlowStateDetailResp) color
    ///
    /// 关联的[修改后状态](super::flow_state_dto::FlowStateDetailResp) color
    pub new_flow_state_color: String,
    /// 修改后状态扩展信息
    pub new_flow_state_ext: FlowStateRelModelExt,
    /// 结束时间
    pub finish_time: Option<DateTime<Utc>>,

    /// 参数列表
    pub vars: Option<HashMap<String, Value>>,
    /// 流转动作列表
    pub next_flow_transitions: Vec<FlowInstFindNextTransitionResp>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstModifyAssignedReq {
    pub current_assigned: String,
}

/// 修改当前参数列表
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstModifyCurrentVarsReq {
    /// 参数列表
    pub vars: HashMap<String, Value>,
}

/// 工作流实例过滤器
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct FlowInstFilterReq {
    /// 关联模型ID
    pub flow_model_id: Option<String>,
    /// 标签
    pub tag: Option<String>,

    /// 是否结束
    pub finish: Option<bool>,
    /// 当前状态ID
    pub current_state_id: Option<String>,
    pub with_sub: Option<bool>,
}

#[derive(sea_orm::FromQueryResult)]
pub struct FlowInstSummaryResult {
    pub id: String,
    pub rel_flow_model_id: String,
    pub rel_flow_model_name: String,

    pub current_vars: Option<Value>,
    pub current_state_id: String,
    pub rel_business_obj_id: String,

    pub create_ctx: Value,
    pub create_time: DateTime<Utc>,

    pub finish_ctx: Option<Value>,
    pub finish_time: Option<DateTime<Utc>>,
    pub finish_abort: Option<bool>,
    pub output_message: Option<String>,

    pub own_paths: String,

    pub tag: String,
}
