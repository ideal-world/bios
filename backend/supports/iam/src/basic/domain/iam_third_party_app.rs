use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// 第三方应用（扩展表，与 rbum_item 通过 id 关联，name 维护在 rbum_item 中）
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "iam_third_party_app")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// 描述
    pub description: Option<String>,
    /// 图标
    pub icon: String,
    /// 链接地址
    pub link_url: String,
    /// 状态
    pub status: i16,
    /// 排序
    pub sort: i64,
    #[fill_ctx(fill = "own_paths")]
    pub own_paths: String,
}
