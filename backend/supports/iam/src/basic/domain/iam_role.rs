use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "iam_role")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub icon: String,
    pub sort: i64,

    pub kind: i16,
    /// 角色权限类型，read 表示只读，all 表示全部权限
    pub perm_kind: String,

    pub in_base: bool,
    pub in_embed: bool,
    pub extend_role_id: String,

    pub deletable: bool,

    #[fill_ctx(fill = "own_paths")]
    pub own_paths: String,
}
