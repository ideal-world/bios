use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Resource domain model
///
/// 资源域模型
///
/// The resource domain is the unit of ownership of the resource, indicating the owner of the resource.
/// Each resource is required to belong to a resource domain.
///
/// 资源域是资源的归属单位，表示资源的所有者。每个资源都要求归属于一个资源域。
///
/// E.g. All menu resources are provided by IAM components, and all IaaS resources are provided by CMDB components.
/// IAM components and CMDB components are resource domains.
///
/// 例如：所有菜单资源由IAM组件提供，所有IaaS资源由CMDB组件提供。IAM组件和CMDB组件是资源域。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_domain")]
pub struct Model {
    /// Resource domain id
    ///
    /// 资源域id
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Resource domain code
    ///
    /// 资源域编码
    ///
    /// Global unique
    ///
    /// 全局唯一
    ///
    /// Which is required to conform to the host specification in the uri, matching the regular: ^[a-z0-9-.]+$.
    ///
    /// 需要符合uri中的host规范，匹配正则：^[a-z0-9-.]+$。
    #[index(unique)]
    pub code: String,
    /// Resource domain name
    ///
    /// 资源域名称
    pub name: String,
    /// Resource domain note
    ///
    /// 资源域备注
    pub note: String,
    /// Resource domain icon
    ///
    /// 资源域图标
    pub icon: String,
    /// Resource domain sort
    ///
    /// 资源域排序
    pub sort: i64,

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
}
