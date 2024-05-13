use serde::{Deserialize, Serialize};
use serde_json::Value;
use tardis::web::poem_openapi::{
    self,
    types::{ParseFromJSON, ToJSON},
};

use super::{flow_state_dto::FlowSysStateKind, flow_transition_dto::FlowTransitionActionByVarChangeInfoChangedKind};

#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowExternalReq {
    /// Type of request initiated, ex: query field, modification field, status change notification...
    /// 
    /// 发起请求的类型，例：查询字段，修改字段，状态变更通知..
    pub kind: FlowExternalKind,
    /// When kind is ModifyField, the field is modified in a specific way, for example: validate the content, post action, precondition trigger ...
    /// 
    /// 当 kind 为 ModifyField 时，字段被修改的具体操作方式，例：验证内容，后置动作，前置条件触发..
    pub callback_op: Option<FlowExternalCallbackOp>,
    /// The tag corresponding to the current business
    /// 
    /// 当前业务对应的 tag
    pub curr_tag: String,
    /// Current Business ID
    /// 
    /// 当前业务ID
    pub curr_bus_obj_id: String,
    /// Workflow Instance ID
    /// 
    /// 工作流实例ID
    pub inst_id: String,
    /// Modified State ID
    /// 
    /// 修改后的状态ID
    pub target_state: Option<String>,
    /// Modified state type
    /// 
    /// 修改后的状态类型
    pub target_sys_state: Option<FlowSysStateKind>,
    /// Status ID before modification
    /// 
    /// 修改前的状态ID
    pub original_state: Option<String>,
    /// Type of state before modification
    /// 
    /// 修改前的状态类型
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
    /// 扩展字段
    /// 
    /// Extended params
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum FlowExternalCallbackOp {
    #[default]
    Default,
    PostAction,
    VerifyContent,
    ConditionalTrigger,
}

#[derive(Debug, Deserialize, Serialize, poem_openapi::Object, Clone)]
pub struct FlowExternalParams {
    pub rel_tag: Option<String>,
    pub rel_kind: Option<String>,
    pub var_id: Option<String>,
    pub var_name: Option<String>,
    pub value: Option<Value>,
    pub changed_kind: Option<FlowTransitionActionByVarChangeInfoChangedKind>,
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
