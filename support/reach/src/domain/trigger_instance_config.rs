use crate::dto::*;
use crate::fill_by_add_req;
use crate::fill_by_mod_req;
use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, DateTime, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;

use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "reach_trigger_instance_config")]
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
    #[sea_orm(column_type = "String(Some(512))")]
    /// 关联的触发场景id
    pub rel_reach_trigger_scene_id: String,
    #[sea_orm(column_type = "String(Some(512))")]
    /// 关联的触达通道
    pub rel_reach_channel: ReachChannelKind,
    #[sea_orm(column_type = "String(Some(512))")]
    /// 关联资源项id
    pub rel_item_id: String,
    #[sea_orm(column_type = "String(Some(255))")]
    /// 接收组编码
    pub receive_group_code: String,
    #[sea_orm(column_type = "String(Some(255))")]
    /// 接收组名称
    pub receive_group_name: String,
}

impl ActiveModelBehavior for ActiveModel {}

impl From<&ReachTriggerInstanceConfigAddReq> for ActiveModel {
    fn from(value: &ReachTriggerInstanceConfigAddReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            create_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_add_req!(value => {
            rel_reach_trigger_scene_id,
            rel_reach_channel,
            rel_item_id,
            receive_group_code,
            receive_group_name,
        } model);
        model
    }
}

impl From<&ReachTriggerInstanceConfigModifyReq> for ActiveModel {
    fn from(value: &ReachTriggerInstanceConfigModifyReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_mod_req!(value => {
            rel_reach_trigger_scene_id,
            rel_reach_channel,
            rel_item_id,
            receive_group_code,
            receive_group_name,
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
            .col(ColumnDef::new(Column::RelReachTriggerSceneId).not_null().string_len(512))
            .col(ColumnDef::new(Column::RelReachChannel).not_null().string_len(512))
            .col(ColumnDef::new(Column::RelItemId).not_null().string_len(512))
            .col(ColumnDef::new(Column::ReceiveGroupCode).not_null().string_len(255))
            .col(ColumnDef::new(Column::ReceiveGroupName).not_null().string_len(255));

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
