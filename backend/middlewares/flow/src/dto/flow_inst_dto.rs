use std::{collections::HashMap, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, error::TardisError},
    chrono::{DateTime, Utc},
    db::sea_orm,
    serde_json::Value,
    web::poem_openapi,
    TardisFuns,
};

use super::{
    flow_state_dto::{FlowGuardConf, FlowStateKind, FlowStateOperatorKind, FlowStateRelModelExt, FlowStateVar, FlowSysStateKind},
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
    /// 触发的动作ID
    pub transition_id: Option<String>,
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
    /// 触发的动作ID
    pub transition_id: Option<String>,
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
    pub rel_flow_version_id: String,
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

    pub tag: String,

    pub main: bool,
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
    /// 当前状态系统类型
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) sys_state
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) 系统类型
    pub current_state_sys_kind: Option<FlowSysStateKind>,

    /// 当前状态类型
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) state_kind
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) 状态类型
    pub current_state_kind: Option<FlowStateKind>,
    /// 当前状态关联扩展信息
    /// Associated [flow_state](super::flow_state_dto::FlowStateRelModelExt)
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateRelModelExt)
    pub current_state_ext: Option<FlowStateRelModelExt>,
    /// Associated [flow_state](super::flow_state_dto::FlowStateRelModelExt)
    ///
    /// 当前状态配置
    pub current_state_conf: Option<FLowInstStateConf>,
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

    pub artifacts: Option<Value>,

    pub own_paths: String,
}

impl FlowInstDetailResp {
    pub fn artifacts(&self) -> FlowInstArtifacts {
        if let Some(artifacts) = self.artifacts.clone() {
            TardisFuns::json.json_to_obj(artifacts).unwrap_or_default()
        } else {
            FlowInstArtifacts::default()
        }
    }
}

// 状态配置
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FLowInstStateConf {
    pub operators: HashMap<FlowStateOperatorKind, String>,
    pub form_conf: Option<FLowInstStateFormConf>,
    pub approval_conf: Option<FLowInstStateApprovalConf>,
}

// 状态录入配置
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FLowInstStateFormConf {
    pub form_vars_collect_conf: HashMap<String, FlowStateVar>,
}

// 状态审批配置
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FLowInstStateApprovalConf {
    pub approval_vars_collect_conf: Option<HashMap<String, FlowStateVar>>,
    pub form_vars_collect: HashMap<String, Value>,
}

// 流程实例中对应的数据存储
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowInstArtifacts {
    pub guard_conf: FlowGuardConf,                                      // 当前操作人权限
    pub approval_result: HashMap<String, HashMap<String, Vec<String>>>, // 当前审批结果
    pub form_state_map: HashMap<String, HashMap<String, Value>>,        // 录入节点映射 key为节点ID,对应的value为节点中的录入的参数
    pub prev_non_auto_state_id: Option<String>,                         // 上一个非自动节点ID
    pub prev_non_auto_account_id: Option<String>,                       // 上一个节点操作人ID
}

// 流程实例中数据存储更新
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default, sea_orm::FromJsonQueryResult)]
pub struct FlowInstArtifactsModifyReq {
    pub guard_conf: Option<FlowGuardConf>,                             // 当前操作人权限
    pub add_guard_conf_account_id: Option<String>,                     // 增加操作人ID
    pub delete_guard_conf_account_id: Option<String>,                  // 删除操作人ID
    pub add_approval_result: Option<(String, FlowApprovalResultKind)>, // 增加审批结果
    pub form_state_map: Option<HashMap<String, Value>>,                // 录入节点映射 key为节点ID,对应的value为节点中的录入的参数
    pub clear_form_result: Option<String>,                             // 清除节点录入信息
    pub clear_approval_result: Option<String>,                         // 清除节点审批信息
    pub prev_non_auto_state_id: Option<String>,                        // 上一个非自动节点ID
    pub prev_non_auto_account_id: Option<String>,                      // 上一个节点操作人ID
}

/// 审批结果类型
#[derive(Serialize, Deserialize, Debug, poem_openapi::Enum, Default, Eq, Hash, PartialEq, Clone)]
pub enum FlowApprovalResultKind {
    /// 通过
    #[default]
    Pass,
    /// 拒绝
    Overrule,
}

impl Display for FlowApprovalResultKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlowApprovalResultKind::Pass => write!(f, "PASS"),
            FlowApprovalResultKind::Overrule => write!(f, "OVERRULE"),
        }
    }
}

impl FromStr for FlowApprovalResultKind {
    type Err = TardisError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "PASS" => Ok(Self::Pass),
            "OVERRULE" => Ok(Self::Overrule),
            _ => Err(TardisError::bad_request(&format!("invalid FlowApprovalResultKind: {}", s), "400-operator-invalid-param")),
        }
    }
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
    /// 目标状态节点 （若未通过transition流转状态，则传入该值）
    pub target_state_id: Option<String>,
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
    pub current_flow_state_sys_kind: FlowSysStateKind,
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
    /// 绑定其他工作流的动作
    pub rel_flow_versions: HashMap<String, String>,
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

/// 操作实例请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowInstOperateReq {
    pub operate: FlowStateOperatorKind,
    /// 参数列表
    pub vars: Option<HashMap<String, Value>>,
    /// 输出信息
    pub output_message: Option<String>,
    /// 操作人
    pub operator: Option<String>,
}

/// 工作流实例过滤器
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct FlowInstFilterReq {
    pub kind: Option<FlowApprovalFilterKind>,
    pub ids: Option<Vec<String>>,
    /// 关联模型ID
    pub flow_version_id: Option<String>,
    /// 业务ID
    pub rel_business_obj_ids: Option<Vec<String>>,
    /// 标签
    pub tag: Option<String>,

    /// 是否主流程
    pub main: Option<bool>,

    /// 是否结束
    pub finish: Option<bool>,
    /// 当前状态ID
    pub current_state_id: Option<String>,
    pub with_sub: Option<bool>,
}

#[derive(sea_orm::FromQueryResult)]
pub struct FlowInstSummaryResult {
    pub id: String,
    pub rel_flow_version_id: String,
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


/// 审批结果类型
#[derive(Serialize, Deserialize, Debug, poem_openapi::Enum, Eq, Hash, PartialEq, Clone)]
pub enum FlowApprovalFilterKind {
    /// 全部
    All,
    /// 待录入
    Form,
    /// 待审批
    Approval,
    /// 我创建的
    Create,
}