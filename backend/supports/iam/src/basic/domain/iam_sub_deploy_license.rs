use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "iam_sub_deploy_license")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    #[index]
    pub sub_deploy_id: String,
    pub license: String,

    pub start_time: chrono::DateTime<Utc>,
    pub end_time: chrono::DateTime<Utc>,

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
