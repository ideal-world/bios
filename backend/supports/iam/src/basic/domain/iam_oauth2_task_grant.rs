use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// 用户对特定应用后台任务的 OAuth 委托；不保存 token。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "iam_oauth2_task_grant")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub task_id: String,
    pub account_id: String,
    pub app_id: String,
    pub provider_grant_id: String,
    pub executor_account_id: String,
    pub revoked: bool,
    pub create_time: String,
    pub own_paths: String,
    #[index(index_id = "current_slot", name = "ux_iam_oauth2_task_grant_current_slot", unique, if_not_exists = true)]
    pub current_own_paths: Option<String>,
    #[index(index_id = "current_slot", unique, if_not_exists = true)]
    pub current_task_id: Option<String>,
    pub revoked_by: Option<String>,
    pub revoked_time: Option<String>,
    pub replaced_by_grant_id: Option<String>,
}
