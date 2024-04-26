use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};
/// Association model for resource set categories(nodes) and resource items
///
/// 资源集分类（节点）挂载资源项的关联模型
///
/// Used to bind resource items to resource set categories(nodes).
///
/// 用于将资源项绑定到资源集分类（节点）。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_set_cate_item")]
pub struct Model {
    /// Association id
    ///
    /// 关联id
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Association sort
    ///
    /// 关联排序
    pub sort: i64,
    /// Associated [resource set](crate::rbum::domain::rbum_set::Model) id
    ///
    /// 关联[资源集](crate::rbum::domain::rbum_set::Model) id
    #[index(index_id = "unique_index", unique)]
    pub rel_rbum_set_id: String,
    /// Associated [resource set category](crate::rbum::domain::rbum_set_cate::Model) sys_code
    ///
    /// 关联[资源集分类](crate::rbum::domain::rbum_set_cate::Model) sys_code
    ///
    /// Avoid recursive tree queries by associating with ``sys_code``.
    ///
    /// 通过 ``sys_code`` 关联以避免递归树查询。
    #[index(index_id = "unique_index")]
    pub rel_rbum_set_cate_code: String,
    /// Associated [resource](crate::rbum::domain::rbum_item::Model) id
    ///
    /// 关联[资源](crate::rbum::domain::rbum_item::Model) id
    #[index(repeat(index_id = "unique_index"))]
    pub rel_rbum_item_id: String,

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
