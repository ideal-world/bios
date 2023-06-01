use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Model / 模型
///
/// Used to define processes, each process contains one or more transitions (associated with `flow_transition`)
/// 用于定义流程，每个流程包含一个或多个流转（关联 `flow_transition` ）
///
/// The current model does not support the version
/// 当前模型不支持版本
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "flow_model")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub icon: String,
    pub info: String,
    /// Initial state / 初始状态
    ///
    /// Define the initial state of each model
    /// 定义每个模块的初始状态
    pub init_state_id: String,
    /// Tags / 标签
    ///
    /// Used for model classification
    /// 用于模型分类
    #[index]
    pub tag: String,

    #[fill_ctx(own_paths)]
    pub own_paths: String,
}
