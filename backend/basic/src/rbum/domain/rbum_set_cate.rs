use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};
/// Resource set category(node) model
///
/// 资源集分类（节点）模型
///
/// Resource set category is essentially a node of the resource tree.
///
/// 资源集分类本质上是资源树的一个个节点。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_set_cate")]
pub struct Model {
    /// Node id
    ///
    /// 节点id
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// System (internal) code
    ///
    /// 系统（内部）编码
    ///
    /// using regular hierarchical code to avoid recursive tree queries.
    ///
    /// 使用规则的层级编码，避免递归树查询。
    #[index(index_id = "unique_sys_code", unique)]
    pub sys_code: String,
    /// Business code for custom
    ///
    /// 自定义业务编码
    #[index(index_id = "bus_code")]
    pub bus_code: String,
    /// Node name
    ///
    /// 节点名称
    pub name: String,
    /// Node icon
    ///
    /// 节点图标
    pub icon: String,
    /// Node sort
    ///
    /// 节点排序
    pub sort: i64,
    /// Node extension information
    ///
    /// 节点扩展信息
    pub ext: String,
    /// Associated [resource set](crate::rbum::domain::rbum_set::Model) id
    ///
    /// 关联[资源集](crate::rbum::domain::rbum_set::Model) id
    #[index(index_id = "unique_sys_code", repeat(index_id = "bus_code"))]
    pub rel_rbum_set_id: String,

    pub scope_level: i16,

    #[fill_ctx(fill = "own_paths")]
    pub own_paths: String,
    #[fill_ctx]
    pub owner: String,
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub create_time: chrono::DateTime<Utc>,
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub update_time: chrono::DateTime<Utc>,
    #[fill_ctx]
    pub create_by: String,
    #[fill_ctx(insert_only = false)]
    pub update_by: String,
}
