use bios_basic::dto::BasicQueryCondInfo;
use serde::{Deserialize, Serialize};
use strum::Display;
use tardis::{
    basic::{error::TardisError, field::TrimString},
    db::sea_orm::{self, EnumIter},
    serde_json::Value,
    web::poem_openapi,
    TardisFuns,
};

use super::flow_var_dto::FlowVarInfo;

/// 添加动作
#[derive(Serialize, Deserialize, Debug, Default, Clone, poem_openapi::Object)]
pub struct FlowTransitionAddReq {
    /// 修改前状态
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_flow_state_id: String,
    /// 修改后状态
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_flow_state_id: String,
    /// 动作名
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: Option<TrimString>,
    /// 为true时，不需要用户干预，在满足条件的前提下自动流转
    pub transfer_by_auto: Option<bool>,
    /// 存在值时，到达时间后，在满足条件的前提下自动流转
    pub transfer_by_timer: Option<String>,
    /// 权限配置：为true时，创建人可以操作
    pub guard_by_creator: Option<bool>,
    /// 权限配置：为true时，历史操作人可以操作
    pub guard_by_his_operators: Option<bool>,
    /// 权限配置：为true时，负责人可以操作
    pub guard_by_assigned: Option<bool>,
    /// 权限配置：为true时，指定操作人可以操作
    pub guard_by_spec_account_ids: Option<Vec<String>>,
    /// 权限配置：为true时，指定角色可以操作
    pub guard_by_spec_role_ids: Option<Vec<String>>,
    /// 权限配置：为true时，指定组织可以操作
    pub guard_by_spec_org_ids: Option<Vec<String>>,
    /// 权限配置：满足条件时，允许操作
    pub guard_by_other_conds: Option<Vec<Vec<BasicQueryCondInfo>>>,
    /// 二次确认配置信息
    pub double_check: Option<FlowTransitionDoubleCheckInfo>,
    /// 验证内容
    pub vars_collect: Option<Vec<FlowVarInfo>>,
    /// 是否通知
    pub is_notify: Option<bool>,
    /// 触发前回调的配置信息
    pub action_by_pre_callback: Option<String>,
    /// 触发后回调的配置信息
    pub action_by_post_callback: Option<String>,
    /// 后置动作的配置信息
    pub action_by_post_changes: Option<Vec<FlowTransitionPostActionInfo>>,
    /// 前置动作的配置信息
    pub action_by_front_changes: Option<Vec<FlowTransitionFrontActionInfo>>,
    /// 排序
    pub sort: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, Default, Clone)]
pub struct FlowTransitionModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub id: TrimString,
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: Option<TrimString>,
    /// 修改前状态
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub from_flow_state_id: Option<String>,
    /// 修改后状态
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub to_flow_state_id: Option<String>,
    /// 为true时，不需要用户干预，在满足条件的前提下自动流转
    pub transfer_by_auto: Option<bool>,
    /// 存在值时，到达时间后，在满足条件的前提下自动流转
    pub transfer_by_timer: Option<String>,
    /// 权限配置：为true时，创建人可以操作
    pub guard_by_creator: Option<bool>,
    /// 权限配置：为true时，历史操作人可以操作
    pub guard_by_his_operators: Option<bool>,
    /// 权限配置：为true时，负责人可以操作
    pub guard_by_assigned: Option<bool>,
    /// 权限配置：为true时，指定操作人可以操作
    pub guard_by_spec_account_ids: Option<Vec<String>>,
    /// 权限配置：为true时，指定角色可以操作
    pub guard_by_spec_role_ids: Option<Vec<String>>,
    /// 权限配置：为true时，指定组织可以操作
    pub guard_by_spec_org_ids: Option<Vec<String>>,
    /// 权限配置：满足条件时，允许操作
    pub guard_by_other_conds: Option<Vec<Vec<BasicQueryCondInfo>>>,
    /// 二次确认配置信息
    pub double_check: Option<FlowTransitionDoubleCheckInfo>,
    /// 验证内容
    pub vars_collect: Option<Vec<FlowVarInfo>>,
    /// 是否通知
    pub is_notify: Option<bool>,
    /// 触发前回调的配置信息
    pub action_by_pre_callback: Option<String>,
    /// 触发后回调的配置信息
    pub action_by_post_callback: Option<String>,
    /// 后置动作的配置信息
    pub action_by_post_changes: Option<Vec<FlowTransitionPostActionInfo>>,
    pub action_by_post_var_changes: Option<Vec<FlowTransitionPostActionInfo>>,
    pub action_by_post_state_changes: Option<Vec<FlowTransitionPostActionInfo>>,
    /// 前置动作的配置信息
    pub action_by_front_changes: Option<Vec<FlowTransitionFrontActionInfo>>,
    /// 排序
    pub sort: Option<i64>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowTransitionDetailResp {
    pub id: String,
    pub name: String,
    /// 修改前状态ID
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) id
    pub from_flow_state_id: String,
    /// 修改前状态名
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) name
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) name
    pub from_flow_state_name: String,
    /// 修改前状态的标示颜色
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) color
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) color
    pub from_flow_state_color: String,
    /// 修改后状态ID
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) id
    pub to_flow_state_id: String,
    /// 修改后状态名
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) name
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) name
    pub to_flow_state_name: String,
    /// 修改后状态的标示颜色
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) color
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) color
    pub to_flow_state_color: String,
    /// 为true时，不需要用户干预，在满足条件的前提下自动流转
    pub transfer_by_auto: bool,
    /// 存在值时，到达时间后，在满足条件的前提下自动流转
    pub transfer_by_timer: String,
    /// 权限配置：为true时，创建人可以操作
    pub guard_by_creator: bool,
    /// 权限配置：为true时，历史操作人可以操作
    pub guard_by_his_operators: bool,
    /// 权限配置：为true时，负责人可以操作
    pub guard_by_assigned: bool,
    /// 权限配置：为true时，指定操作人可以操作
    pub guard_by_spec_account_ids: Vec<String>,
    /// 权限配置：为true时，指定角色可以操作
    pub guard_by_spec_role_ids: Vec<String>,
    /// 权限配置：为true时，指定组织可以操作
    pub guard_by_spec_org_ids: Vec<String>,
    /// 权限配置：满足条件时，允许操作
    // TODO
    pub guard_by_other_conds: Value,
    /// 验证内容
    pub vars_collect: Value,
    /// 二次确认配置信息
    pub double_check: Value,
    /// 是否通知
    pub is_notify: bool,
    /// 触发前回调的配置信息
    pub action_by_pre_callback: String,
    /// 触发后回调的配置信息
    pub action_by_post_callback: String,
    /// 后置动作的配置信息
    pub action_by_post_changes: Value,
    /// 前置动作的配置信息
    pub action_by_front_changes: Value,
    /// Associated [flow_state](super::flow_model_dto::FlowModelDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_model_dto::FlowModelDetailResp) id
    pub rel_flow_model_id: String,
    /// 排序
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

    pub fn action_by_post_changes(&self) -> Vec<FlowTransitionPostActionInfo> {
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
            is_notify: Some(value.is_notify),
            sort: Some(value.sort),
        }
    }
}

/// 二次确认
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowTransitionDoubleCheckInfo {
    /// 是否开启弹窗
    pub is_open: bool,
    /// 提示内容
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

/// 后置动作配置信息
#[derive(Serialize, Deserialize, Clone, PartialEq, Default, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowTransitionPostActionInfo {
    /// 后置动作类型，目前有状态修改和字段修改两种。
    pub kind: FlowTransitionActionChangeKind,
    /// 描述
    pub describe: String,
    /// 对象Tag。为空时，代表触发的业务对象为当前操作的业务对象。有值时，代表触发的业务对象为当前操作的业务对象的关联对象。
    /// 例：值为req，则代表触发的业务对象为当前操作对象所关联的需求对象。
    pub obj_tag: Option<String>,
    /// 对象tag的关联类型。当 kind 为 State 时该字段生效，当 kind 为 var 时该字段统一为 default。
    /// 目前有默认和父子关系两种。为空时，代表是默认关联类型。当值为 Default 时，obj_tag 为 req/issue/test/task等。当值为 ParentOrSub 时，obj_tag 为 parent/sub.
    /// 例：当值为 ParentOrSub，obj_tag 为 parent。表示为当前操作对象所关联的父级对象。
    pub obj_tag_rel_kind: Option<TagRelKind>,
    /// 筛选当前状态符合的业务对象。
    pub obj_current_state_id: Option<Vec<String>>,
    /// 触发动作所需要关联对象符合的条件
    pub change_condition: Option<StateChangeCondition>,
    /// 当 kind 为 State 时生效，触发修改的目标状态
    pub changed_state_id: String,
    /// 当 kind 为 Var 时生效，为 false 表示修改当前操作对象字段，为 true 时，表示修改当前操作对象的关联对象字段
    pub current: bool,
    /// 被修改的字段名
    pub var_name: String,
    /// 修改内容
    pub changed_val: Option<Value>,
    /// 修改方式（清空，更改内容，更改为其他字段的值，加减值等）
    pub changed_kind: Option<FlowTransitionActionByVarChangeInfoChangedKind>,

    /// 是否可修改（前端用于判断当前配置是否可编辑）
    pub is_edit: Option<bool>,
}

impl From<FlowTransitionPostActionInfo> for FlowTransitionActionChangeAgg {
    fn from(value: FlowTransitionPostActionInfo) -> Self {
        match value.kind {
            FlowTransitionActionChangeKind::State => FlowTransitionActionChangeAgg {
                kind: value.kind,
                var_change_info: None,
                state_change_info: Some(FlowTransitionActionByStateChangeInfo {
                    obj_tag: value.obj_tag.unwrap(),
                    obj_tag_rel_kind: value.obj_tag_rel_kind,
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
                    obj_tag_rel_kind: value.obj_tag_rel_kind,
                    var_name: value.var_name,
                    changed_val: value.changed_val,
                    changed_kind: value.changed_kind,
                }),
                state_change_info: None,
            },
        }
    }
}

/// 后置动作聚合信息
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object)]
pub struct FlowTransitionActionChangeAgg {
    /// 后置动作类型，目前有状态修改和字段修改两种。
    pub kind: FlowTransitionActionChangeKind,
    /// 字段修改配置信息
    pub var_change_info: Option<FlowTransitionActionByVarChangeInfo>,
    /// 状态变更配置信息
    pub state_change_info: Option<FlowTransitionActionByStateChangeInfo>,
}

/// 后置动作类型，目前有状态修改和字段修改两种。
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default, Serialize, poem_openapi::Enum, EnumIter, sea_orm::DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "FlowTransitionActionChangeKind")]
pub enum FlowTransitionActionChangeKind {
    /// 字段修改
    #[default]
    #[sea_orm(string_value = "var")]
    Var,
    /// 状态变更
    #[sea_orm(string_value = "state")]
    State,
}

/// 字段修改配置信息
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowTransitionActionByVarChangeInfo {
    /// 当 kind 为 Var 时生效，为 false 表示修改当前操作对象字段，为 true 时，表示修改当前操作对象的关联对象字段
    pub current: bool,
    /// 描述
    pub describe: String,
    /// 对象Tag。为空时，代表触发的业务对象为当前操作的业务对象。有值时，代表触发的业务对象为当前操作的业务对象的关联对象。
    /// 例：值为req，则代表触发的业务对象为当前操作对象所关联的需求对象。
    pub obj_tag: Option<String>,
    /// 对象tag的关联类型。当 kind 为 State 时该字段有效，当 kind 为 var 时该字段统一为 default。
    /// 目前有默认和父子关系两种。为空时，代表是默认关联类型。当值为 Default 时，obj_tag 为 req/issue/test/task等。当值为 ParentOrSub 时，obj_tag 为 parent/sub.
    /// 例：当值为 ParentOrSub，obj_tag 为 parent。表示为当前操作对象所关联的父级对象。
    pub obj_tag_rel_kind: Option<TagRelKind>,
    /// 被修改的字段名
    pub var_name: String,
    /// 修改内容
    pub changed_val: Option<Value>,
    /// 修改方式（清空，更改内容，更改为其他字段的值，加减值等）
    pub changed_kind: Option<FlowTransitionActionByVarChangeInfoChangedKind>,
}

/// 修改方式（清空，更改内容，更改为其他字段的值，加减值等）
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, EnumIter, sea_orm::DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "FlowTransitionActionByVarChangeInfoChangedKind")]
pub enum FlowTransitionActionByVarChangeInfoChangedKind {
    /// 清空
    #[sea_orm(string_value = "clean")]
    Clean,
    /// 更改为指定内容
    #[sea_orm(string_value = "change_content")]
    ChangeContent,
    /// 更改为当前时间（仅支持时间格式的字段）
    #[sea_orm(string_value = "auto_get_operate_time")]
    AutoGetOperateTime,
    /// 更改为当前操作人（仅支持用户相关值得字段）
    #[sea_orm(string_value = "auto_get_operator")]
    AutoGetOperator,
    /// 更改为指定字段值（仅支持相同类型字段）
    #[sea_orm(string_value = "select_field")]
    SelectField,
    /// 加某个值或者减某个值（仅支持数值类字段）
    #[sea_orm(string_value = "and_or_subs")]
    AddOrSub,
}

/// 状态变更的配置信息
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowTransitionActionByStateChangeInfo {
    /// 对象Tag。为空时，代表触发的业务对象为当前操作的业务对象。有值时，代表触发的业务对象为当前操作的业务对象的关联对象。
    /// 例：值为req，则代表触发的业务对象为当前操作对象所关联的需求对象。
    pub obj_tag: String,
    /// 对象tag的关联类型。当 kind 为 State 时该字段生效，当 kind 为 var 时该字段统一为 default。
    /// 目前有默认和父子关系两种。为空时，代表是默认关联类型。当值为 Default 时，obj_tag 为 req/issue/test/task等。当值为 ParentOrSub 时，obj_tag 为 parent/sub.
    /// 例：当值为 ParentOrSub，obj_tag 为 parent。表示为当前操作对象所关联的父级对象。
    pub obj_tag_rel_kind: Option<TagRelKind>,
    /// 描述
    pub describe: String,
    /// 筛选当前状态符合的业务对象。
    pub obj_current_state_id: Option<Vec<String>>,
    /// 触发动作所需要关联对象符合的条件
    pub change_condition: Option<StateChangeCondition>,
    /// 修改的目标状态
    pub changed_state_id: String,
}

/// 触发动作所需要关联对象符合的条件
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct StateChangeCondition {
    /// 是否启用该条件规则
    pub current: bool,
    /// 筛选条件的配置项
    pub conditions: Vec<StateChangeConditionItem>,
}

/// 筛选条件的配置项
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct StateChangeConditionItem {
    /// 对象Tag。为空时，代表触发的业务对象为当前操作的业务对象。有值时，代表触发的业务对象为当前操作的业务对象的关联对象。
    /// 例：值为req，则代表触发的业务对象为当前操作对象所关联的需求对象。
    pub obj_tag: Option<String>,
    /// 对象tag的关联类型。当 kind 为 State 时该字段生效，当 kind 为 var 时该字段统一为 default。
    /// 目前有默认和父子关系两种。为空时，代表是默认关联类型。当值为 Default 时，obj_tag 为 req/issue/test/task等。当值为 ParentOrSub 时，obj_tag 为 parent/sub.
    /// 例：当值为 ParentOrSub，obj_tag 为 parent。表示为当前操作对象所关联的父级对象。
    pub obj_tag_rel_kind: Option<TagRelKind>,
    /// 需要符合的状态ID
    pub state_id: Vec<String>,
    /// 实际规则的条件类型
    pub op: StateChangeConditionOp,
}

/// 实际规则的条件类型
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Enum)]
pub enum StateChangeConditionOp {
    And,
    Or,
}

/// 对象tag的关联类型。当 kind 为 State 时该字段生效，当 kind 为 var 时该字段统一为 default。
/// 目前有默认和父子关系两种。为空时，代表是默认关联类型。当值为 Default 时，obj_tag 为 req/issue/test/task等。当值为 ParentOrSub 时，obj_tag 为 parent/sub.
/// 例：当值为 ParentOrSub，obj_tag 为 parent。表示为当前操作对象所关联的父级对象。
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Enum)]
pub enum TagRelKind {
    Default,
    ParentOrSub,
}

impl From<TagRelKind> for String {
    fn from(kind: TagRelKind) -> Self {
        match kind {
            TagRelKind::Default => "DEFAULT".to_string(),
            TagRelKind::ParentOrSub => "PARENT_OR_SUB".to_string(),
        }
    }
}

#[derive(Default)]
pub struct FlowTransitionInitInfo {
    pub from_flow_state_id: String,
    pub to_flow_state_id: String,
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
    pub is_notify: bool,

    pub action_by_pre_callback: Option<String>,
    pub action_by_post_callback: Option<String>,
    pub action_by_post_changes: Vec<FlowTransitionPostActionInfo>,
    pub action_by_front_changes: Vec<FlowTransitionFrontActionInfo>,

    pub sort: Option<i64>,
}

impl TryFrom<FlowTransitionInitInfo> for FlowTransitionAddReq {
    type Error = TardisError;

    fn try_from(value: FlowTransitionInitInfo) -> Result<Self, Self::Error> {
        Ok(FlowTransitionAddReq {
            from_flow_state_id: value.from_flow_state_id,
            to_flow_state_id: value.to_flow_state_id,
            name: Some(value.name.into()),
            is_notify: Some(true),
            transfer_by_auto: value.transfer_by_auto,
            transfer_by_timer: value.transfer_by_timer,
            guard_by_creator: value.guard_by_creator,
            guard_by_his_operators: value.guard_by_his_operators,
            guard_by_assigned: value.guard_by_assigned,
            guard_by_spec_account_ids: value.guard_by_spec_account_ids,
            guard_by_spec_role_ids: value.guard_by_spec_role_ids,
            guard_by_spec_org_ids: value.guard_by_spec_org_ids,
            guard_by_other_conds: value.guard_by_other_conds,
            vars_collect: value.vars_collect,
            action_by_pre_callback: value.action_by_pre_callback,
            action_by_post_callback: value.action_by_post_callback,
            action_by_post_changes: Some(value.action_by_post_changes),
            action_by_front_changes: Some(value.action_by_front_changes),
            double_check: value.double_check,
            sort: value.sort,
        })
    }
}

/// 前置条件配置信息
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowTransitionFrontActionInfo {
    /// 关联关系类型
    pub relevance_relation: FlowTransitionFrontActionInfoRelevanceRelation,
    /// 关联关系名
    pub relevance_label: String,
    /// 左值
    pub left_value: String,
    /// 左值字段名
    pub left_label: String,
    /// 右值类型（某个字段，某个值，当前时间等）
    pub right_value: FlowTransitionFrontActionRightValue,
    /// 当right_value为SelectField时生效，选择字段。
    pub select_field: Option<String>,
    /// 当right_value为SelectField时生效，选择字段名。
    pub select_field_label: Option<String>,
    /// 当right_value为ChangeContent时生效，填写值。
    pub change_content: Option<Value>,
    /// 当right_value为ChangeContent时生效，填写值的标签。
    pub change_content_label: Option<String>,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum FlowTransitionFrontActionInfoRelevanceRelation {
    #[serde(rename = "=")]
    #[oai(rename = "=")]
    Eq,
    #[serde(rename = "!=")]
    #[oai(rename = "!=")]
    Ne,
    #[serde(rename = ">")]
    #[oai(rename = ">")]
    Gt,
    #[serde(rename = ">=")]
    #[oai(rename = ">=")]
    Ge,
    #[serde(rename = "<")]
    #[oai(rename = "<")]
    Lt,
    #[serde(rename = "<=")]
    #[oai(rename = "<=")]
    Le,
    #[serde(rename = "like")]
    #[oai(rename = "like")]
    Like,
    #[serde(rename = "not_like")]
    #[oai(rename = "not_like")]
    NotLike,
    #[serde(rename = "in")]
    #[oai(rename = "in")]
    In,
    #[serde(rename = "not_in")]
    #[oai(rename = "not_in")]
    NotIn,
    #[serde(rename = "between")]
    #[oai(rename = "between")]
    Between,
}

impl FlowTransitionFrontActionInfoRelevanceRelation {
    pub fn check_conform(&self, left_value: String, right_value: String) -> bool {
        use itertools::Itertools;

        if left_value.is_empty() || left_value == "null" || right_value == "null" {
            return false;
        }
        match self {
            FlowTransitionFrontActionInfoRelevanceRelation::Eq => left_value == right_value,
            FlowTransitionFrontActionInfoRelevanceRelation::Ne => left_value != right_value,
            FlowTransitionFrontActionInfoRelevanceRelation::Gt => left_value > right_value,
            FlowTransitionFrontActionInfoRelevanceRelation::Ge => left_value >= right_value,
            FlowTransitionFrontActionInfoRelevanceRelation::Lt => left_value < right_value,
            FlowTransitionFrontActionInfoRelevanceRelation::Le => left_value <= right_value,
            FlowTransitionFrontActionInfoRelevanceRelation::Like => left_value.contains(&right_value),
            FlowTransitionFrontActionInfoRelevanceRelation::NotLike => !left_value.contains(&right_value),
            FlowTransitionFrontActionInfoRelevanceRelation::In => TardisFuns::json
                .str_to_obj::<Vec<Value>>(&right_value)
                .unwrap_or_default()
                .into_iter()
                .map(|item| item.as_str().unwrap_or(item.to_string().as_str()).to_string())
                .collect_vec()
                .contains(&left_value),
            FlowTransitionFrontActionInfoRelevanceRelation::NotIn => !TardisFuns::json
                .str_to_obj::<Vec<Value>>(&right_value)
                .unwrap_or_default()
                .into_iter()
                .map(|item| item.as_str().unwrap_or(item.to_string().as_str()).to_string())
                .collect_vec()
                .contains(&left_value),
            FlowTransitionFrontActionInfoRelevanceRelation::Between => {
                let interval = TardisFuns::json.str_to_obj::<Vec<String>>(&right_value).unwrap_or_default();
                if interval.len() != 2 {
                    return false;
                }
                left_value >= interval[0] && left_value <= interval[1]
            }
        }
    }
}

/// 右值类型
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, poem_openapi::Enum)]
#[serde(rename_all = "snake_case")]
pub enum FlowTransitionFrontActionRightValue {
    /// 某个字段
    #[oai(rename = "select_field")]
    SelectField,
    /// 某个值
    #[oai(rename = "change_content")]
    ChangeContent,
    /// 当前时间
    #[oai(rename = "real_time")]
    RealTime,
}
