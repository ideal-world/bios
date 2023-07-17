use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, DateTime, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;

use tardis::db::sea_orm::sea_query::{ColumnDef, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

use crate::dto::{ReachDndStrategyKind, ReachMsgLogAddReq, ReachMsgLogModifyReq};
use crate::fill_by_add_req;
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "reach_msg_log")]
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
    /// 关联接收人Id
    pub rel_account_id: String,
    /// 免扰时间，HH::MM-HH:MM
    pub dnd_time: String,
    /// 免扰策略
    pub dnd_strategy: ReachDndStrategyKind,
    /// 开始时间
    pub start_time: chrono::DateTime<Utc>,
    /// 结束时间
    pub end_time: chrono::DateTime<Utc>,
    /// 完成时间
    pub finish_time: Option<chrono::DateTime<Utc>>,
    /// 是否失败
    pub failure: bool,
    /// 失败原因
    pub fail_message: String,
    /// 用户触达消息Id
    pub rel_reach_message_id: String,
}

impl From<&ReachMsgLogAddReq> for ActiveModel {
    fn from(value: &ReachMsgLogAddReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            create_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_add_req!(
            value => {
                rel_account_id,
                dnd_time,
                dnd_strategy,
                start_time,
                end_time,
                failure,
                fail_message,
                rel_reach_message_id,
            } model
        );

        model
    }
}

impl From<&ReachMsgLogModifyReq> for ActiveModel {
    fn from(value: &ReachMsgLogModifyReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_add_req!(
            value => {
                rel_account_id,
                dnd_time,
                dnd_strategy,
                start_time,
                end_time,
                failure,
                fail_message,
            } model
        );
        model
    }
}
impl ActiveModelBehavior for ActiveModel {}

impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(ctx.own_paths.to_string());
            self.owner = Set(ctx.owner.to_string());
        }
    }

    fn create_table_statement(db: DbBackend) -> TableCreateStatement {
        let mut builder = Table::create();
        builder
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            // Specific
            .col(ColumnDef::new(Column::RelAccountId).not_null().string())
            .col(ColumnDef::new(Column::DndTime).not_null().string())
            .col(ColumnDef::new(Column::DndStrategy).not_null().string())
            .col(ColumnDef::new(Column::StartTime).timestamp())
            .col(ColumnDef::new(Column::EndTime).timestamp())
            .col(ColumnDef::new(Column::FinishTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp())
            .col(ColumnDef::new(Column::Failure).not_null().boolean())
            .col(ColumnDef::new(Column::FailMessage).not_null().string())
            .col(ColumnDef::new(Column::RelReachMessageId).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string());
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
        builder.to_owned()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}