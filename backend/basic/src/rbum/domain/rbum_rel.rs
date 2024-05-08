use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};
/// Resource relationship model
/// 
/// 资源关联模型
/// 
/// Used to describe the relationship between different resources.
/// 
/// 用于描述不同资源间的关联关系。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_rel")]
pub struct Model {
    /// Relationship id
    /// 
    /// 关联id
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Relationship tag
    /// 
    /// 关联标签
    /// 
    /// Used to distinguish different relationships.
    /// 
    /// 用于区分不同的关联关系。
    #[index(index_id = "from_index", repeat(index_id = "to_index"))]
    pub tag: String,
    /// Relationship note
    ///
    /// 关联备注
    pub note: String,
    /// Relationship source type ([`crate::rbum::rbum_enumeration::RbumRelFromKind`])
    /// 
    /// 关联来源方的类型（[`crate::rbum::rbum_enumeration::RbumRelFromKind`]）
    #[index(index_id = "from_index")]
    pub from_rbum_kind: i16,
    /// Relationship source id
    /// 
    /// 关联来源方的id
    #[index(index_id = "from_index")]
    pub from_rbum_id: String,
    /// Relationship target id
    /// 
    /// 关联目标方的id
    #[index(index_id = "to_index")]
    pub to_rbum_item_id: String,
    /// Relationship target ownership path
    /// 
    /// 关联目标方的所有权路径
    pub to_own_paths: String,
    /// Relationship extension information
    ///
    /// 关联扩展信息
    /// 
    /// E.g. the record from or to is in another service, to avoid remote calls,
    /// you can redundantly add the required information to this field.
    /// 
    /// 例如：记录来源或目标在另一个服务中，为避免远程调用，可以将所需信息冗余添加到此字段。
    pub ext: String,

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
