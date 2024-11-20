use std::collections::HashMap;

use bios_basic::{
    dto::BasicQueryCondInfo,
    rbum::{
        dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher, RbumItemRelFilterReq},
        rbum_enumeration::RbumScopeLevelKind,
    },
};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    db::sea_orm::{self, prelude::*, EnumIter},
    serde_json::Value,
    web::poem_openapi,
    TardisFuns,
};

use super::flow_transition_dto::FlowTransitionDetailResp;

#[derive(Clone, Serialize, Deserialize, Default, Debug, poem_openapi::Object)]
pub struct FlowStateAddReq {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub id: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub id_prefix: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sys_state: FlowSysStateKind,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub info: Option<String>,
    pub state_kind: Option<FlowStateKind>,
    pub kind_conf: Option<FLowStateKindConf>,

    pub template: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_state_id: Option<String>,

    #[oai(validator(min_length = "2", max_length = "200"))]
    pub tags: Option<Vec<String>>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, Default, Clone)]
pub struct FLowStateKindConf {
    pub form: Option<FlowStateForm>,
    pub approval: Option<FlowStateApproval>,
}

/// 录入节点配置信息
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, Default, Clone)]
pub struct FlowStateForm {
    pub nodify: bool,
    pub nodify_conf: Option<FlowNodifyConf>,
    /// 权限配置：为true时，创建人可以操作
    pub guard_by_creator: bool,
    /// 权限配置：为true时，历史操作人可以操作
    pub guard_by_his_operators: bool,
    /// 权限配置：为true时，负责人可以操作
    pub guard_by_assigned: bool,
    /// 权限配置：自定义配置
    pub guard_custom: bool,
    /// 权限配置：自定义配置
    pub guard_custom_conf: Option<FlowGuardConf>,
    /// 当操作人为空时的自动处理策略
    pub auto_transfer_when_empty_kind: Option<FlowStatusAutoStrategyKind>,
    /// 当操作人为空且策略选择为指定代理，则当前配置人员权限生效
    pub auto_transfer_when_empty_guard_custom_conf: Option<FlowGuardConf>,
    /// 当操作人为空且策略选择为流转节点，则当前配置节点ID生效
    pub auto_transfer_when_empty_state_id: Option<String>,
    /// 是否允许转办
    pub referral: bool,
    /// 转办自定义人员权限
    pub referral_guard_custom_conf: Option<FlowGuardConf>,
    /// 字段配置
    pub vars_collect: HashMap<String, FlowStateVar>,
}

/// 审批节点配置信息
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, Default, Clone)]
pub struct FlowStateApproval {
    /// 通知
    pub nodify: bool,
    pub nodify_conf: Option<FlowNodifyConf>,
    /// 结果通知
    pub response_nodify: bool,
    pub response_nodify_conf: Option<FlowNodifyConf>,
    /// 权限配置：为true时，创建人可以操作
    pub guard_by_creator: bool,
    /// 权限配置：为true时，历史操作人可以操作
    pub guard_by_his_operators: bool,
    /// 权限配置：为true时，负责人可以操作
    pub guard_by_assigned: bool,
    /// 权限配置：自定义配置
    pub guard_custom: bool,
    /// 权限配置：自定义配置
    pub guard_custom_conf: Option<FlowGuardConf>,
    /// 当操作人为空时的自动处理策略
    pub auto_transfer_when_empty_kind: Option<FlowStatusAutoStrategyKind>,
    /// 当操作人为空且策略选择为指定代理，则当前配置人员权限生效
    pub auto_transfer_when_empty_guard_custom_conf: Option<FlowGuardConf>,
    /// 当操作人为空且策略选择为流转节点，则当前配置节点ID生效
    pub auto_transfer_when_empty_state_id: Option<String>,
    /// 是否允许撤销
    pub revoke: bool,
    /// 是否允许转办
    pub referral: bool,
    /// 转办自定义人员权限
    pub referral_guard_custom: bool,
    pub referral_guard_custom_conf: Option<FlowGuardConf>,
    /// 字段配置
    pub vars_collect: HashMap<String, FlowStateVar>,
    /// 多人审批策略方式
    pub multi_approval_kind: FlowStatusMultiApprovalKind,
    /// 会签配置
    pub countersign_conf: FlowStateCountersignConf,
}

/// 状态节点字段配置
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, Default, Clone)]
pub struct FlowStateVar {
    pub show: bool,
    pub edit: bool,
    pub required: bool,
}

/// 状态自动处理的策略类型
#[derive(Serialize, Deserialize, Debug, poem_openapi::Enum, Default, EnumIter, sea_orm::DeriveActiveEnum, Clone)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum FlowStatusAutoStrategyKind {
    /// 自动跳过
    #[default]
    #[sea_orm(string_value = "autoskip")]
    Autoskip,
    /// 指定代理
    #[sea_orm(string_value = "specify_agent")]
    SpecifyAgent,
    /// 流转节点
    #[sea_orm(string_value = "transfer_state")]
    TransferState,
}

/// 多人审批策略方式
#[derive(Serialize, Deserialize, Debug, poem_openapi::Enum, Default, EnumIter, sea_orm::DeriveActiveEnum, Clone)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum FlowStatusMultiApprovalKind {
    /// 或签
    #[default]
    #[sea_orm(string_value = "orsign")]
    Orsign,
    /// 会签
    #[sea_orm(string_value = "countersign")]
    Countersign,
}

/// 会签配置
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, Default, Clone)]
pub struct FlowStateCountersignConf {
    /// 类型
    pub kind: FlowStateCountersignKind,
    /// 多数人通过比例
    pub most_percent: Option<i8>,
    /// 审批人权限配置
    pub guard_custom_conf: Option<FlowGuardConf>,
    /// 指定人通过即通过
    pub specified_pass_guard: Option<bool>,
    pub specified_pass_guard_conf: Option<FlowGuardConf>,
    /// 指定人拒绝即拒绝
    pub specified_overrule_guard: Option<bool>,
    pub specified_overrule_guard_conf: Option<FlowGuardConf>,
}

/// 多人审批策略方式
#[derive(Serialize, Deserialize, Debug, poem_openapi::Enum, Default, EnumIter, sea_orm::DeriveActiveEnum, Clone)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum FlowStateCountersignKind {
    /// 所有人签
    #[default]
    #[sea_orm(string_value = "all")]
    All,
    /// 多数人签
    #[sea_orm(string_value = "most")]
    Most,
}

/// 人员权限配置
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, Default, Clone)]
pub struct FlowGuardConf {
    /// 权限配置：为true时，指定操作人可以操作
    pub guard_by_spec_account_ids: Vec<String>,
    /// 权限配置：为true时，指定角色可以操作
    pub guard_by_spec_role_ids: Vec<String>,
    /// 权限配置：为true时，指定组织可以操作
    pub guard_by_spec_org_ids: Vec<String>,
}

// 节点通知配置
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, Default, Clone)]
pub struct FlowNodifyConf {
    /// 权限配置：指定操作人可以操作
    pub guard_by_owner: bool,
    /// 权限配置：自定义配置
    pub guard_custom: bool,
    /// 权限配置：自定义配置
    pub guard_custom_conf: Option<FlowGuardConf>,
    /// 通知方式：短信通知
    pub send_sms: bool,
    /// 通知方式：邮箱通知
    pub send_mail: bool,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, Default, Clone)]
pub struct FlowStateModifyReq {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sys_state: Option<FlowSysStateKind>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub info: Option<String>,
    pub state_kind: Option<FlowStateKind>,
    pub kind_conf: Option<FLowStateKindConf>,

    pub template: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_state_id: Option<String>,

    #[oai(validator(min_length = "2", max_length = "200"))]
    pub tags: Option<Vec<String>>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowStateSummaryResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub sys_state: FlowSysStateKind,
    pub info: String,

    pub template: bool,
    pub rel_state_id: String,

    pub tags: String,

    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub disabled: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowStateDetailResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub sys_state: FlowSysStateKind,
    pub info: String,

    pub state_kind: FlowStateKind,
    pub kind_conf: Value,

    pub template: bool,
    pub rel_state_id: String,

    pub tags: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}

impl FlowStateDetailResp {
    pub fn kind_conf(&self) -> FLowStateKindConf {
        TardisFuns::json.json_to_obj(self.kind_conf.clone()).unwrap()
    }
}

/// Type of state
///
/// 状态类型
// #[sea_orm(rs_type = "String", db_type = "String(Some(255))")]
#[derive(Clone, Default, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, EnumIter, sea_orm::DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum FlowSysStateKind {
    #[default]
    #[sea_orm(string_value = "start")]
    Start,
    #[sea_orm(string_value = "progress")]
    Progress,
    #[sea_orm(string_value = "finish")]
    Finish,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, EnumIter, sea_orm::DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum FlowStateKind {
    /// 普通节点
    #[default]
    #[sea_orm(string_value = "simple")]
    Simple,
    /// 录入节点
    #[sea_orm(string_value = "form")]
    Form,
    #[sea_orm(string_value = "mail")]
    Mail,
    #[sea_orm(string_value = "callback")]
    Callback,
    #[sea_orm(string_value = "timer")]
    Timer,
    #[sea_orm(string_value = "script")]
    Script,
    /// 审批节点
    #[sea_orm(string_value = "approval")]
    Approval,
    /// 分支节点
    #[sea_orm(string_value = "branch")]
    Branch,
    /// 开始节点
    #[sea_orm(string_value = "start")]
    Start,
    /// 结束节点
    #[sea_orm(string_value = "finish")]
    Finish,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct FlowStateFilterReq {
    pub basic: RbumBasicFilterReq,
    pub sys_state: Option<FlowSysStateKind>,
    pub tag: Option<String>,
    pub state_kind: Option<FlowStateKind>,
    pub template: Option<bool>,
    pub flow_version_ids: Option<Vec<String>>,
}

impl RbumItemFilterFetcher for FlowStateFilterReq {
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

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowStateNameResp {
    pub key: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowStateCountGroupByStateReq {
    pub tag: String,
    pub inst_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowStateCountGroupByStateResp {
    pub state_name: String,
    pub count: String,
    pub inst_ids: Vec<String>,
}

#[derive(Serialize, Clone, Deserialize, Debug, Default, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowStateRelModelExt {
    pub sort: i64,
    pub show_btns: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object, sea_orm::FromQueryResult, Clone)]
pub struct FlowStateRelModelModifyReq {
    pub id: String,
    pub sort: Option<i64>,
    pub show_btns: Option<Vec<String>>,
}

/// 工作流状态聚合信息
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct FlowStateAggResp {
    pub id: String,
    pub name: String,
    pub is_init: bool,
    pub ext: FlowStateRelModelExt,
    pub state_kind: FlowStateKind,
    pub kind_conf: Value,
    pub sys_state: FlowSysStateKind,
    pub tags: String,
    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
    pub transitions: Vec<FlowTransitionDetailResp>,
}

impl FlowStateAggResp {
    pub fn kind_conf(&self) -> FLowStateKindConf {
        TardisFuns::json.json_to_obj(self.kind_conf.clone()).unwrap()
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct FLowStateIdAndName {
    pub id: String,
    pub name: String,
}
