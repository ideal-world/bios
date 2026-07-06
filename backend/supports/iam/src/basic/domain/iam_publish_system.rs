use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// 发布系统（扩展表，与 rbum_item 通过 id 关联，系统名称维护在 rbum_item 中）
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "iam_publish_system")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// 系统标识
    #[index]
    pub sys_ident: Option<String>,
    /// 描述
    pub description: Option<String>,
    #[fill_ctx(fill = "own_paths")]
    pub own_paths: String,
}
