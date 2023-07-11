use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc, DateTime};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::Uuid;
use tardis::db::sea_orm::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use crate::dto::{ReachStatusKind, ReachMsgSignatureAddReq, ReachMsgSignatureModifyReq};

use crate::dto::ReachChannelKind;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "reach_msg_signature")]
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
    /// 名称
    #[sea_orm(column_name = "name", column_type = "String(Some(255))")]
    pub name: String,    
    /// 说明
    #[sea_orm(column_name = "note", column_type = "String(Some(2000))")]
    pub note: String,
    /// 内容
    #[sea_orm(column_name = "content", column_type = "String(Some(2000))")]
    pub content: String,
    /// 来源
    #[sea_orm(column_name = "source", column_type = "String(Some(255))")]
    pub source: String,
    /// 关联的触达通道
    #[sea_orm(column_name = "source", column_type = "String(Some(255))")]
    pub rel_reach_channel: ReachChannelKind,
}

impl From<&ReachMsgSignatureAddReq> for ActiveModel {
    fn from(value: &ReachMsgSignatureAddReq) -> Self {
        ActiveModel {
            name: Set(value.name.clone()),
            note: Set(value.note.clone()),
            content: Set(value.content.clone()),
            source: Set(value.source.clone()),
            rel_reach_channel: Set(value.rel_reach_channel),
            create_time: Set(chrono::Utc::now()),
            update_time: Set(chrono::Utc::now()),
            ..Default::default()
        }
    }
}

impl From<&ReachMsgSignatureModifyReq> for ActiveModel {
    fn from(value: &ReachMsgSignatureModifyReq) -> Self {
        let mut model = ActiveModel {
            create_time: Set(chrono::Utc::now()),
            update_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        if let Some(name) = &value.name {
            model.name = Set(name.clone());
        }
        if let Some(note) = &value.note {
            model.note = Set(note.clone());
        }
        if let Some(content) = &value.content {
            model.content = Set(content.clone());
        }
        if let Some(source) = &value.source {
            model.source = Set(source.clone());
        }
        if let Some(rel_reach_channel) = &value.rel_reach_channel {
            model.rel_reach_channel = Set(*rel_reach_channel);
        }
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
        builder.table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            .col(ColumnDef::new(Column::Name).not_null().string())
            .col(ColumnDef::new(Column::Note).not_null().string())
            .col(ColumnDef::new(Column::Content).not_null().string())
            .col(ColumnDef::new(Column::Source).not_null().string())
            .col(ColumnDef::new(Column::RelReachChannel).not_null().string())
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            .col(ColumnDef::new(Column::CreateTime).timestamp())
            .col(ColumnDef::new(Column::UpdateTime).timestamp());
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