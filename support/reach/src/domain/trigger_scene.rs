use crate::dto::*;
use crate::fill_by_add_req;

use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, DateTime, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;

use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "reach_trigger_scene")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, generator = "uuid")]
    pub id: String,
    /// 所有者路径
    #[sea_orm(column_name = "own_paths", column_type = "String(Some(255))")]
    pub own_paths: String,
    /// 所有者
    #[sea_orm(column_name = "owner", column_type = "String(Some(255))")]
    pub owner: String,
    /// 创建时间
    #[sea_orm(column_name = "create_time", column_type = "Timestamp")]
    pub create_time: DateTime<Utc>,
    /// 更新时间
    #[sea_orm(column_name = "update_time", column_type = "Timestamp")]
    pub update_time: DateTime<Utc>,
    #[sea_orm(column_name = "update_time", column_type = "String(Some(255))")]
    /// 编码
    pub code: String,
    #[sea_orm(column_name = "update_time", column_type = "String(Some(255))")]
    /// 名称
    pub name: String,
    #[sea_orm(column_name = "update_time", column_type = "String(Some(2000))")]
    /// 父场景ID
    pub pid: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}

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

impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.owner = Set(ctx.owner.to_string());
            self.own_paths = Set(ctx.own_paths.to_string());
        }
    }
    fn create_table_statement(db: DbBackend) -> TableCreateStatement {
        let mut builder = Table::create();
        builder
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            .col(ColumnDef::new(Column::Code).not_null().string_len(255))
            .col(ColumnDef::new(Column::Name).not_null().string_len(255))
            .col(ColumnDef::new(Column::Pid).not_null().string_len(2000));

        if db == DatabaseBackend::Postgres {
            builder
                .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp_with_time_zone())
                .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp_with_time_zone());
        } else {
            builder
                .engine("InnoDB")
                .character_set("utf8mb4")
                .collate("utf8mb4_0900_as_cs")
                .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp())
                .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).timestamp());
        }
        builder
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![]
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
