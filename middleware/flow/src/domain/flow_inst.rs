use crate::dto::flow_inst_dto::FlowOperationContext;
use tardis::chrono::Utc;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::Json;
use tardis::db::sea_orm::*;
use tardis::{chrono, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Process instance / 流程实例
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "flow_inst")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[index]
    pub rel_flow_model_id: String,

    /// Business object Id / 关联的业务对象Id
    #[index(unique = true)]
    pub rel_business_obj_id: String,

    /// Current state / 当前状态
    ///
    /// This state needs to be updated after each transfer
    /// 每次流转后都需要更新此状态
    #[index]
    pub current_state_id: String,
    /// Current variable list / 当前变量列表
    ///
    /// This variable list needs to be updated after each transfer
    /// 每次流转后都需要更新此变量列表
    /// TODO Vec<FlowVarSimpleInfo>
    pub current_vars: Option<Json>,

    /// Variable list when created / 创建时的变量列表（HashMap<String, Value>）
    pub create_vars: Option<Json>,
    /// Creator information / 创建者信息
    pub create_ctx: FlowOperationContext,
    /// Creation time / 创建时间
    #[index]
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub create_time: chrono::DateTime<Utc>,

    /// Finisher information / 完成者信息
    pub finish_ctx: Option<FlowOperationContext>,
    /// Finish time / 完成时间
    ///
    /// When this value exists, it means it has been completed
    /// 在存在此值时，表示已完成
    #[index]
    pub finish_time: Option<chrono::DateTime<Utc>>,
    /// Whether to be aborted / 是否被中止
    pub finish_abort: Option<bool>,
    /// Output message when finished / 完成时的输出信息
    pub output_message: Option<String>,

    /// Transfer information list  / 流转信息列表
    ///
    /// TODO Vec<FlowInstTransitionInfo>
    #[index(full_text)]
    #[sea_orm(column_type = "JsonBinary", nullable)]
    #[tardis_entity(custom_type = "JsonBinary")]
    pub transitions: Option<Json>,

    pub own_paths: String,

    /// Current Assigned  / 指定执行人
    pub current_assigned: Option<String>,
}
