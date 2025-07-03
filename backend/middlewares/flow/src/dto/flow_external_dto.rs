use serde::{Deserialize, Serialize};
use serde_json::Value;
use tardis::web::poem_openapi::{
    self,
    types::{ParseFromJSON, ToJSON},
};

use super::{
    flow_state_dto::{FlowGuardConf, FlowSysStateKind},
    flow_transition_dto::FlowTransitionActionByVarChangeInfoChangedKind,
};

/// External data exchange requests
///
/// 对外数据交换请求
#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowExternalReq {
    /// Associated [enum](FlowExternalKind)
    ///
    /// 关联的[枚举](FlowExternalKind)
    pub kind: FlowExternalKind,
    /// Associated [enum](FlowExternalCallbackOp)
    ///
    /// 关联的[枚举](FlowExternalCallbackOp)
    pub callback_op: Option<FlowExternalCallbackOp>,
    /// The tag corresponding to the current business
    ///
    /// 当前业务对应的 tag
    pub curr_tag: String,
    /// Current Business ID
    ///
    /// 当前业务ID
    pub curr_bus_obj_id: String,
    /// Associated [flow_instance](super::flow_inst_dto::FlowInstDetailResp) id
    ///
    /// 关联的[工作流实例](super::flow_inst_dto::FlowInstDetailResp) id
    pub inst_id: String,
    /// Modified State ID
    ///
    /// 修改后的状态ID
    pub target_state: Option<String>,
    /// Associated [enum](super::flow_state_dto::FlowSysStateKind)
    ///
    /// 关联的[枚举](super::flow_state_dto::FlowSysStateKind)
    pub target_sys_state: Option<FlowSysStateKind>,
    /// Status ID before modification
    ///
    /// 修改前的状态ID
    pub original_state: Option<String>,
    /// Associated [enum](super::flow_state_dto::FlowSysStateKind)
    ///
    /// 关联的[枚举](super::flow_state_dto::FlowSysStateKind)
    pub original_sys_state: Option<FlowSysStateKind>,
    /// Name of the action actually triggered (business side logging operation)
    ///
    /// 实际触发的动作名称（业务方记录操作日志）
    pub transition_name: Option<String>,
    pub owner_paths: String,
    /// When kind is QueryField, batch pass business IDs
    ///
    /// 当 kind 为 QueryField 时，批量传入业务ID
    pub obj_ids: Vec<String>,
    /// Whether the request triggers a notification
    ///
    /// 请求是否触发通知
    pub notify: Option<bool>,
    /// Whether the request triggers a notification
    ///
    /// 是否人工操作
    pub manual_op: Option<bool>,
    /// Whether the request triggers a notification
    ///
    /// 操作人
    pub operator: Option<String>,
    /// 触发时间（毫秒时间戳）
    pub sys_time: Option<i64>,
    /// 扩展字段
    ///
    /// Extended params
    pub params: Vec<FlowExternalParams>,
    /// 权限配置
    ///
    /// guard Config
    pub guard_conf: Option<FlowGuardConf>,
}

/// Type of request initiated, ex: query field, modification field, status change notification...
///
/// 发起请求的类型，例：查询字段，修改字段，状态变更通知..
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum FlowExternalKind {
    #[default]
    /// 获取关联业务对象
    FetchRelObj,
    /// 修改字段
    ModifyField,
    /// 状态变更通知
    NotifyChanges,
    /// 查询字段值
    QueryField,
    /// 删除业务对象
    DeleteObj,
    /// 获取权限用户
    FetchAuthAccount,
    /// 更新关联关系
    UpdateRelationship,
}

/// When kind is ModifyField, the field is modified in a specific way, for example: validate the content, post action, precondition trigger ...
///
/// 当 kind 为 ModifyField 时，字段被修改的具体操作方式，例：验证内容，后置动作，前置条件触发..
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum FlowExternalCallbackOp {
    #[default]
    Default,
    /// 后置动作
    PostAction,
    /// 验证内容
    VerifyContent,
    /// 条件触发
    ConditionalTrigger,
    /// 自动流转
    Auto,
}

/// 扩展字段
///
/// Extended params
#[derive(Debug, Deserialize, Serialize, poem_openapi::Object, Clone, Default)]
pub struct FlowExternalParams {
    /// 关联的 Tag
    pub rel_tag: Option<String>,
    /// 关联类型 TagRelKind
    pub rel_kind: Option<String>,
    /// 字段ID
    pub var_id: Option<String>,
    /// 字段名
    pub var_name: Option<String>,
    /// 修改成的值
    pub value: Option<Value>,
    /// 修改方式
    pub changed_kind: Option<FlowTransitionActionByVarChangeInfoChangedKind>,
    /// 权限配置
    pub guard_conf: Option<FlowGuardConf>,
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
    /// 当前Tag
    pub curr_tag: String,
    /// 当前业务ID
    pub curr_bus_obj_id: String,
    /// 关联业务对象
    pub rel_bus_objs: Vec<RelBusObjResp>,
}

#[derive(Default, Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct RelBusObjResp {
    /// 关联对象的Tag
    pub rel_tag: String,
    /// 关联业务对象ID
    pub rel_bus_obj_ids: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalModifyFieldResp {}

#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowExternalNotifyChangesResp {}

#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowExternalQueryFieldResp {
    pub objs: Vec<Value>,
}

#[derive(Default, Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalDeleteRelObjResp {}

#[derive(Default, Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalUpdateRelationshipResp {}

#[derive(Default, Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalFetchAuthAccountResp {
    pub account_ids: Vec<String>,
}
