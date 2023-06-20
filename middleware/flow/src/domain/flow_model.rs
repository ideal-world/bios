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

    /// External Data Interaction Interface / 外部的数据交互接口
    /// 
    /// Request Method: PUT
    /// 
    /// Request Context-Type: application/Json
    /// 
    /// ## Get related information
    /// ```
    /// Request Body:{
    ///     "kind": "", // FETCH_REL_OBJ
    ///     "curr_tag": "", // 当前类型，对应于此模型的 `tag` 字段
    ///     "curr_bus_obj_id": "", // 当前业务对象Id
    ///     "fetch_rel_obj": {
    ///         "rel_tag": "", // 关联类型，对应于此模型的 `tag` 字段
    ///         "rel_state_ids": [""] // 关联状态Id，可选
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
    pub exchange_data_url: String,

    #[fill_ctx(own_paths)]
    pub own_paths: String,
}
