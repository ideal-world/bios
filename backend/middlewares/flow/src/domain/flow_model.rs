use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::Json;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

use crate::dto::flow_model_dto::{FlowModelKind, FlowModelStatus};

/// Model / 模型
///
/// Used to define processes, each process contains one or more transitions (associated with `flow_transition`)
/// 用于定义流程，每个流程包含一个或多个流转（关联 `flow_transition` ）
///
/// The current model does not supports the version
/// 当前模型不支持版本
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "flow_model")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub icon: String,
    pub info: String,

    /// Model variable list / 模型变量列表
    pub vars: Option<Json>,

    /// Types of workflow models / 工作流模型类型
    /// 此功能用于标记工作流模型的类型，目前有仅作为模板，仅作为实例，既可作为模板又可作为实例三种。表示当前模型的用途和功能。
    #[tardis_entity(custom_type = "String")]
    pub kind: FlowModelKind,

    /// Status of workflow models / 工作流模型状态
    /// 启用/停用
    #[tardis_entity(custom_type = "String")]
    pub status: FlowModelStatus,

    ///  Associated template / 关联模板
    ///
    /// his function is used to associate this template with other templates, e.g. if the template refers to a template, then this association corresponds to the Id of the template
    /// 此功能用于将该模型与模板关联，比如该模型引用于某个模板，则此关联对应于模板的Id
    pub rel_template_id: Option<String>,

    ///  Currently enabled version ID / 当前启用的版本ID
    ///
    /// This field is used to record the version of the model currently in use
    /// 此字段用于记录当前模型在使用的版本
    pub current_version_id: String,

    /// Whether it is a mainstream process / 是否是主流程
    ///
    #[index]
    pub main: bool,
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
    /// 此功能用于将该模型与其他模型关联，比如该模型引用于某个模型，则此关联对应于模型的Id
    #[index]
    pub rel_model_id: String,
    /// Tags / 标签
    ///
    /// Used for model classification
    /// 用于模型分类
    #[index]
    #[tardis_entity(custom_type = "String")]
    pub tag: Option<String>,

    /// External Data Interaction Interface / 外部的数据交互接口
    ///
    /// Request Method: PUT
    ///
    /// Request Context-Type: application/json
    ///
    /// ## Get related information
    /// ```
    /// Request Body:{
    ///     "kind": "", // FETCH_REL_OBJ
    ///     "curr_tag": "", // 当前类型，对应于此模型的 `tag` 字段
    ///     "curr_bus_obj_id": "", // 当前业务对象Id
    ///     "fetch_rel_obj": {
    ///         "rel_tag": "", // 关联类型，对应于此模型的 `tag` 字段
    ///         "rel_curr_state_ids": [""] // 关联状态Id，可选
    ///         "rel_changed_state": "" // 关联变更的状态，可选
    ///     }
    /// }
    ///
    /// Response Body: {
    ///     "code": "200",
    ///     "msg": "",
    ///     "data": [{
    ///         "rel_bus_obj_id": "" // 关联的业务对象Id
    ///     }]
    /// }
    ///
    /// ## 变更通知
    ///
    /// Request Body:{
    ///     "kind": "", // NOTIFY_CHANGES
    ///     "curr_tag": "", // 当前类型，对应于此模型的 `tag` 字段
    ///     "curr_bus_obj_id": "", // 当前业务对象Id
    ///     "notify_changes": {
    ///         "rel_tag": "", // 关联类型，对应于此模型的 `tag` 字段
    ///         "rel_bus_obj_id": "", // 关联的业务对象Id
    ///         "changed_vars": {} // 变更的变量列表
    ///     }
    /// }
    ///
    /// Response Body: {
    ///     "code": "200",
    ///     "msg": "",
    ///     "data": {}
    /// }
    /// ```
    // pub exchange_data_url: String,

    #[fill_ctx(fill = "own_paths")]
    pub own_paths: String,
}
