
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
    pub name: String,
    pub rel_model_id: String,
    pub current_version_sign: String,
    pub status: String,
    pub create_time: chrono::DateTime<Utc>,
    pub create_by: String,
    pub update_time: chrono::DateTime<Utc>,
    pub update_by: String,
}