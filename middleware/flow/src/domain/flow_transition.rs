use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::Json;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Transfer / 流转
///
/// Used to define the flow of the process (migration of state)
/// 用于定义流程的流转（状态的迁移）
///
/// Some conditions (`guard_xx`) need to be satisfied when moving from one state to another, multiple conditions are `OR` relations
/// 从某个状态转到另一个状态时需要满足一些条件（ `guard_xx` ）,多个条件为 `OR` 关系。
///
/// guard similar to `Gateway` in BPMN
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "flow_transition")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    /// Source state / 来源状态
    pub from_flow_state_id: String,
    /// Target state / 目标状态
    pub to_flow_state_id: String,

    /// Automatic transfer / 自动流转
    ///
    /// When true, no user intervention is required, and the transfer is automatic under the premise of meeting the conditions
    /// 为true时，不需要用户干预，在满足条件的前提下自动流转
    /// TODO 该功能未实现
    pub transfer_by_auto: bool,
    /// Timed transfer / 定时流转
    ///
    /// The value is the number of seconds of delay
    /// 值为延时的秒数
    ///
    /// When there is a value, after the time is reached, it will be automatically transferred under the premise of meeting the conditions
    /// 存在值时，到达时间后，在满足条件的前提下自动流转
    // TODO 该功能未实现
    pub transfer_by_timer: String,

    /// Transfer condition: the current operator is the creator
    /// 流转条件：当前操作人是创建者
    pub guard_by_creator: bool,
    /// Transfer condition: the current operator is a historical operator
    /// 流转条件：当前操作人是历史操作人
    pub guard_by_his_operators: bool,
    /// Transfer condition: the current operator contains the corresponding users
    /// 流转条件：当前操作人包含对应的用户
    pub guard_by_spec_account_ids: Vec<String>,
    /// Transfer condition: he current operator contains the corresponding roles
    /// 流转条件：当前操作人包含对应的角色
    pub guard_by_spec_role_ids: Vec<String>,
    /// Transfer condition: the condition that the current variable satisfies
    /// 流转条件：当前变量满足的条件
    ///
    /// This conditional format is json wrapped in two layers of arrays:
    /// [ -- The outer layer is an OR relationship
    ///   [{},{}] -- The inner layer is an AND relationship
    /// ]
    pub guard_by_other_conds: Json,

    /// List of variables to be collected / 需要采集的变量列表
    ///
    /// List of variables to be captured when entering this transition
    /// 当进入此流转时，需要采集的变量列表
    /// TODO Vec<FlowVarInfo>
    pub vars_collect: Json,

    /// External interface to be called when entering this transition
    /// 进入此流转时，需要调用的外部接口
    ///
    /// action similar to `Event` in BPMN
    pub action_by_pre_callback: String,
    /// External interface to be called when leaving this transition
    /// 离开此流转时，需要调用的外部接口
    ///
    /// action similar to `Event` in BPMN
    pub action_by_post_callback: String,

    pub rel_flow_model_id: String,

    #[fill_ctx(own_paths)]
    pub own_paths: String,
}
