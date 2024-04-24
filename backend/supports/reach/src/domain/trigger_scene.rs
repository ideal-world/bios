use crate::dto::*;
use crate::fill_by_add_req;

use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, DateTime, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "reach_trigger_scene")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[tardis_entity(custom_type = "string")]
    pub id: Nanoid,
    /// 所有者路径
    #[fill_ctx(fill = "own_paths")]
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub own_paths: String,
    /// 所有者
    #[fill_ctx]
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub owner: String,
    /// 创建时间
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub create_time: DateTime<Utc>,
    /// 更新时间
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub update_time: DateTime<Utc>,
    #[tardis_entity(custom_type = "string", custom_len = "225")]
    /// 编码
    pub code: String,
    #[tardis_entity(custom_type = "string", custom_len = "225")]
    /// 名称
    pub name: String,
    #[tardis_entity(custom_len = "2000")]
    /// 父场景ID
    pub pid: Option<String>,
}

impl From<&ReachTriggerSceneAddReq> for ActiveModel {
    fn from(value: &ReachTriggerSceneAddReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            create_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_add_req!(value => {
            pid,
        } model);
        model.code = Set(value.rbum_add_req.code.as_ref().map(ToString::to_string).unwrap_or_default());
        model.name = Set(value.rbum_add_req.name.to_string());
        model
    }
}

impl From<&ReachTriggerSceneModifyReq> for ActiveModel {
    fn from(value: &ReachTriggerSceneModifyReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_add_req!(value => {
            code,
            name,
        } model);
        model
    }
}
