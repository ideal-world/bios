use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Resource item extended attribute value model
///
/// 资源项扩展属性值模型
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_item_attr")]
pub struct Model {
    /// Extended attribute value id
    ///
    /// 扩展属性值id
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Extended attribute value
    ///
    /// 扩展属性值
    pub value: String,
    /// Associated [resource item](crate::rbum::domain::rbum_item::Model) id
    ///
    /// 关联的[资源项](crate::rbum::domain::rbum_item::Model) id
    #[index(index_id = "id", unique)]
    pub rel_rbum_item_id: String,
    /// Associated [resource kind attribute definition](crate::rbum::domain::rbum_kind_attr::Model) id
    ///
    /// 关联的[资源类型属性定义](crate::rbum::domain::rbum_kind_attr::Model) id
    #[index(index_id = "id", unique)]
    pub rel_rbum_kind_attr_id: String,

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
