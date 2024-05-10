use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Resource item model
///
/// 资源项模型
///
/// Used to represent a specific resource,
/// Each resource item corresponds to a [resource kind](crate::rbum::domain::rbum_kind::Model)  and [resource domain](crate::rbum::domain::rbum_domain::Model).
///
/// 用于表示具体的资源，每个资源项对应一个[资源类型](crate::rbum::domain::rbum_kind::Model) 和 [资源域](crate::rbum::domain::rbum_domain::Model)。
///
/// Each resource item corresponds to a unique uri,
/// and the uri consists of ``<resource kind code>://<resource domain code>/<resource item code>`` .
///
/// 每个资源项对应一个唯一的uri，uri 由 ``<资源类型编码>://<资源域编码>/<资源项编码>`` 组成。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_item")]
pub struct Model {
    /// Resource item id
    ///
    /// 资源项id
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Resource item code
    ///
    /// 资源项编码
    #[index(index_id = "unique_id", unique)]
    pub code: String,
    /// Resource item name
    ///
    /// 资源项名称
    pub name: String,
    /// Associated [resource kind](crate::rbum::domain::rbum_kind::Model) id
    ///
    /// 关联的[资源类型](crate::rbum::domain::rbum_kind::Model) id
    #[index(repeat(index_id = "unique_id", unique))]
    pub rel_rbum_kind_id: String,
    /// Associated [resource domain](crate::rbum::domain::rbum_domain::Model) id
    ///
    /// 关联的[资源域](crate::rbum::domain::rbum_domain::Model) id
    #[index(repeat(index_id = "unique_id", unique))]
    pub rel_rbum_domain_id: String,

    pub scope_level: i16,

    #[index]
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

    pub disabled: bool,
}
