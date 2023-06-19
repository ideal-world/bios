use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

use crate::dto::flow_model_dto::FlowTagKind;

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

    /// Whether it is a template / 是否是模板
    ///
    /// Used as a model for the model to be reused in the process
    /// 用于将该模型作为模板，以便于在流程中复用
    ///
    #[index]
    pub template: bool,
    ///  Associated model / 关联模型
    ///
    /// his function is used to associate this model with other models, e.g. if the model refers to a template, then this association corresponds to the Id of the template
    /// 此功能用于将该模型与其他模型关联，比如该模型引用于某个模板，则此关联对应于模板的Id
    #[index]
    pub rel_model_id: String,
    /// Tags / 标签
    ///
    /// Used for model classification
    /// 用于模型分类
    #[index]
    #[tardis_entity(custom_type = "String")]
    pub tag: Option<FlowTagKind>,

    #[fill_ctx(own_paths)]
    pub own_paths: String,
}
