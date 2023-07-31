use crate::dto::*;
use crate::fill_by_add_req;
use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, DateTime, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;

use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "reach_vcode_strategy")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Nanoid,
    /// 所有者路径
    #[sea_orm(column_type = "String(Some(255))")]
    pub own_paths: String,
    /// 所有者
    #[sea_orm(column_type = "String(Some(255))")]
    pub owner: String,
    /// 创建时间
    #[sea_orm(column_type = "Timestamp")]
    pub create_time: DateTime<Utc>,
    /// 更新时间
    #[sea_orm(column_type = "Timestamp")]
    pub update_time: DateTime<Utc>,
    #[sea_orm(column_type = "TinyInteger")]
    pub max_error_times: i32,
    #[sea_orm(column_type = "SmallInteger")]
    pub expire_sec: i32,
    #[sea_orm(column_type = "TinyInteger")]
    pub length: i32,
    #[sea_orm(column_type = "String(Some(255))")]
    pub rel_reach_set_id: String,
}

impl From<&ReachVCodeStrategyAddReq> for ActiveModel {
    fn from(value: &ReachVCodeStrategyAddReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            create_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_add_req!(
            value => {
                max_error_times,
                expire_sec,
                length,
                rel_reach_set_id,
            } model
        );
        model
    }
}

impl From<&ReachVCodeStrategyModifyReq> for ActiveModel {
    fn from(value: &ReachVCodeStrategyModifyReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_add_req!(
            value => {
                max_error_times,
                expire_sec,
                length,
            } model
        );
        model
    }
}
impl ActiveModelBehavior for ActiveModel {}
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
            .col(ColumnDef::new(Column::MaxErrorTimes).not_null().tiny_integer())
            .col(ColumnDef::new(Column::ExpireSec).not_null().small_integer())
            .col(ColumnDef::new(Column::Length).not_null().tiny_integer())
            .col(ColumnDef::new(Column::MaxErrorTimes).not_null().string_len(255));

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
