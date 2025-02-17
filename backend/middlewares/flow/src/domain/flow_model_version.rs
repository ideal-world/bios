use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{
    chrono::{self, Utc},
    db::sea_orm::DeriveEntityModel,
    TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation,
};

use crate::dto::flow_model_version_dto::FlowModelVesionState;

/// Model Version / 模型版本
///
/// Used to define processes, each process contains one or more transitions (associated with `flow_transition`)
/// 用于定义流程，每个流程包含一个或多个流转（关联 `flow_transition` ）
///
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "flow_model_version")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// 关联的模型ID
    #[index]
    pub rel_model_id: String,
    /// Initial state / 初始状态
    ///
    /// Define the initial state of each model
    /// 定义每个模块的初始状态
    pub init_state_id: String,
    /// 状态 启用中 已关闭
    #[tardis_entity(custom_type = "String")]
    pub status: FlowModelVesionState,
    /// Creation time / 创建时间
    #[index]
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub create_time: chrono::DateTime<Utc>,
    /// 创建者信息
    pub create_by: String,
    /// 更新时间
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub update_time: chrono::DateTime<Utc>,
    /// 修改人信息
    pub update_by: String,
    /// 发布时间
    pub publish_time: Option<chrono::DateTime<Utc>>,
    /// 发布人信息
    pub publish_by: Option<String>,

    #[fill_ctx(fill = "own_paths")]
    pub own_paths: String,
}
