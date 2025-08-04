use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Resource kind model
///
/// 资源类型模型
///
/// A resource kind is a set of common resources.
/// E.g. `/tenant/**` , `/app/**` these are all APIs, and these are all API-kind resources; `/tenant/list` ,
/// `/tenant/detail#more` these are all menus, and these are all  menu-kind resources.
///
/// 资源类型是一组共同的资源。
/// 例如 `/tenant/**` , `/app/**` 这些都是API，这些都是API类型的资源； `/tenant/list` , `/tenant/detail#more` 这些都是菜单，这些都是菜单类型的资源。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_kind")]
pub struct Model {
    /// Resource kind id
    ///
    /// 资源类型id
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Resource kind module
    ///
    /// 资源类型模块
    ///
    /// Used to further divide the resource  kind. For example, there are multiple resource  kinds under the ``cmdb compute`` module, such as ``ecs, ec2, k8s``.
    ///
    /// 用于对资源类型做简单的分类。比如 ``cmdb计算`` 模块下可以有 ``ecs、ec2、k8s`` 等多个资源类型。
    pub module: String,
    /// Resource kind code
    ///
    /// 资源类型编码
    ///
    /// Which is required to conform to the scheme specification in the uri, matching the regular: ``^[a-z0-9-.]+$`` .
    ///
    /// 需要符合uri中的scheme规范，匹配正则：``^[a-z0-9-.]+$`` 。
    #[index(unique)]
    pub code: String,
    /// Resource kind name
    ///
    /// 资源类型名称
    pub name: String,
    /// Resource kind note
    ///
    /// 资源类型备注
    pub note: String,
    /// Resource kind icon
    ///
    /// 资源类型图标
    pub icon: String,
    /// Resource kind sort
    ///
    /// 资源类型排序
    pub sort: i64,
    /// Extension table name
    ///
    /// 扩展表名
    ///
    /// Each resource kind can specify an extension table for storing customized data.
    ///
    /// 每个资源类型可以指定一个扩展表用于存储自定义数据。
    pub ext_table_name: String,

    /// Parent kind id
    ///
    /// 资源类型父id
    #[index]
    pub parent_id: String,

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
