use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

/// Event Topic model
/// 
/// 事件主题模型
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "event_topic")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Whether to save messages
    /// 
    /// 是否保存消息
    pub save_message: bool,
    /// Whether a management node is required
    /// 
    /// 是否需要管理节点
    pub need_mgr: bool,
    pub queue_size: i32,
    /// If need_mgr is false, this field is used when registering
    /// 
    /// 如果 need_mgr 为 false，则在注册时使用该sk
    pub use_sk: String,
    /// If need_mgr is true, this field is used when registering
    ///
    /// 如果 need_mgr 为 true，则在注册时使用该sk
    pub mgr_sk: String,

    pub own_paths: String,
}

impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(ctx.own_paths.to_string());
        }
    }

    fn create_table_statement(db: DbBackend) -> TableCreateStatement {
        let mut builder = Table::create();
        builder
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            .col(ColumnDef::new(Column::SaveMessage).not_null().boolean())
            .col(ColumnDef::new(Column::NeedMgr).not_null().boolean())
            .col(ColumnDef::new(Column::QueueSize).not_null().integer())
            .col(ColumnDef::new(Column::UseSk).not_null().string())
            .col(ColumnDef::new(Column::MgrSk).not_null().string())
            .col(ColumnDef::new(Column::OwnPaths).not_null().string());
        if db == DatabaseBackend::MySql {
            builder.engine("InnoDB").character_set("utf8mb4").collate("utf8mb4_0900_as_cs");
        }
        builder.to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
