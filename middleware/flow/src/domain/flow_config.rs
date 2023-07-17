use tardis::chrono::Utc;
use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{chrono, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// config / 配置
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "flow_config")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[index]
    pub code: String,
    pub name: String,
    pub value: String,

    /// Creation time / 创建时间
    #[index]
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub create_time: chrono::DateTime<Utc>,

    /// Updated time / 更新时间
    #[index]
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub update_time: chrono::DateTime<Utc>,

    #[fill_ctx(own_paths)]
    pub own_paths: String,
}
