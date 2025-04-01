use tardis::chrono::Utc;
use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{chrono, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// IAM Sub Deploy Hosts
///
/// IAM 二级部署访问的黑/白名单
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "iam_sub_deploy_host")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[index]
    pub sub_deploy_id: String,
    pub name: String,
    pub host: String,
    /// 访问类型
    /// [data type Kind](crate::iam_enumeration::IamSubDeployHostKind)
    #[index]
    pub host_type: String,

    pub note: String,

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
