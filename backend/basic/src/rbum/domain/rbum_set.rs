use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Resource set model
///
/// 资源集模型
///
/// Resource set is essentially a general tree structure processing model.
///
/// 资源集本质上是一个通用的树形结构处理模型。
///
/// ```text
/// +------------------------------------+
/// |                                    |
/// |  rbum_set                          |
/// |                                    |
/// |    1|                              |
/// |     |                              |
/// |     +--- * rbum_set_cate           |
/// |                                    |
/// |             1|                     |
/// |              |                     |
/// |              +--- * rbum_set_item  |
/// |                                    |
/// | General Tree        1|             |
/// +----------------------+-------------+
///                        |               
///                        +--- * rbum_item
/// ```
///
/// * ``rbum_set`` is the tree description
/// * ``rbum_set_cate`` is the tree nodes
/// * ``rbum_set_item`` is the association between tree nodes and mounted resource items
///     (supports multiple resource items mounted on a node, and a resource item mounted on multiple nodes)
///
///
/// * ``rbum_set`` 是树的描述
/// * ``rbum_set_cate`` 是树的各个节点
/// * ``rbum_set_item`` 是树节点与挂载资源项的关联（支持一个节点挂载多个资源项，一个资源项挂载多个节点）
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_set")]
pub struct Model {
    /// Resource set id
    ///
    /// 资源集id
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Resource set code
    ///
    /// 资源集编码
    #[index(unique)]
    pub code: String,
    /// Resource set kind
    ///
    /// 资源集类型
    #[index]
    pub kind: String,
    /// Resource set name
    ///
    /// 资源集名称
    pub name: String,
    /// Resource set note
    ///
    /// 资源集备注
    pub note: String,
    /// Resource set icon
    ///
    /// 资源集图标
    pub icon: String,
    /// Resource set sort
    ///
    /// 资源集排序
    pub sort: i64,
    /// Resource set extension information
    ///
    /// 资源集扩展信息
    pub ext: String,

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
