use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "iam_sub_deploy")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// 部署省份
    #[index]
    pub province: String,
    // 访问地址
    pub access_url: String,

    pub note: String,

    // 扩展部署ID
    #[index]
    pub extend_sub_deploy_id: String,

    #[fill_ctx(fill = "own_paths")]
    pub own_paths: String,
}
