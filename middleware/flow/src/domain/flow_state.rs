use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::Json;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

use crate::dto::flow_state_dto::{FlowStateKind, FlowSysStateKind};

/// State / 状态
///
/// Similar to `Task` in BPMN
///
/// The current state does not support the version
/// 当前状态不支持版本
///
/// When modifying key information (E.g. disabled/sys_state/state_kind/kind_conf) or deleting it, modification or deletion is not allowed if the state is already referenced
/// 修改关键信息（E.g. disabled/sys_state/state_kind/kind_conf）或删除时，如果该状态已经被引用，则不允许修改或删除。
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "flow_state")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub icon: String,
    pub info: String,
    /// System state type / 系统状态类型
    ///
    /// Each state should be marked with the corresponding system state
    /// 每个状态都应该标明对应的系统状态
    ///
    /// When the process flows to the system state `Finish`, the process is automatically ended
    /// 当流程流转到系统状态为 `Finish` 时即自动结束流程
    #[index]
    #[tardis_entity(custom_type = "String")]
    pub sys_state: FlowSysStateKind,
    /// State type / 状态类型
    ///
    /// Each state corresponds to a state type
    /// 每个状态都对应于一个状态类型
    ///
    /// Different state types correspond to different predefined behaviors
    /// 不同的状态类型对应于不同的预定义的行为
    ///  E.g.
    /// Simple: Do anything
    /// Form: Fill in the form
    /// Mail: Send an email
    /// Callback: Callback url
    /// Script: Execute a script
    /// ......
    #[index]
    #[tardis_entity(custom_type = "String")]
    pub state_kind: FlowStateKind,
    /// Status type configuration / 状态类型配置
    ///
    /// Different states can correspond to different configuration information
    /// 不同的状态可对应于不同的配置信息
    pub kind_conf: Json,
    /// Whether it is a template / 是否是模板
    ///
    /// Used as a template for the state to be reused in the process
    /// 用于将该状态作为模板，以便于在流程中复用
    ///
    /// TODO 该功能未实现
    #[index]
    pub template: bool,
    ///  Associated state / 关联状态
    ///
    /// his function is used to associate this state with other states, e.g. if the state refers to a template, then this association corresponds to the Id of the template
    /// 此功能用于将该状态与其他状态关联，比如该状态引用于某个模板，则此关联对应于模板的Id
    #[index]
    pub rel_state_id: String,
    /// Tags / 标签
    ///
    /// Used for state classification
    /// 用于状态分类
    #[index]
    pub tag: String,

    #[fill_ctx(own_paths)]
    pub own_paths: String,
}
