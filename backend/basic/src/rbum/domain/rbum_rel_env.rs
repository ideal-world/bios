use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Resource relationship environment condition model
///
/// 资源关联环境条件模型
///
/// This model is used to further qualify the conditions under which the relationship is established.
///
/// 该模型用于进一步限定建立关联关系的条件。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_rel_env")]
pub struct Model {
    /// Relationship environment id
    ///
    /// 关联环境id
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Relationship environment type ([`crate::rbum::rbum_enumeration::RbumRelEnvKind`])
    ///
    /// 关联的环境类型 （[`crate::rbum::rbum_enumeration::RbumRelEnvKind`]）
    pub kind: i16,
    /// Relationship environment value1
    ///
    /// 关联环境值1
    pub value1: String,
    /// Relationship environment value2
    ///
    /// 关联环境值2
    pub value2: String,
    /// Associated [relationship](crate::rbum::domain::rbum_rel::Model) id
    ///
    /// 关联的[资源关联](crate::rbum::domain::rbum_rel::Model) id
    #[index]
    pub rel_rbum_rel_id: String,

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
