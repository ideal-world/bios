use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "iam_account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub icon: String,
    pub temporary: bool,
    // 索引扩展字段 idx 1-3
    pub ext1_idx: String,
    pub ext2_idx: String,
    pub ext3_idx: String,
    // 普通扩展字段 4-9
    pub ext4: String,
    pub ext5: String,
    pub ext6: String,
    pub ext7: String,
    pub ext8: String,
    pub ext9: String,

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
            .col(ColumnDef::new(Column::Icon).not_null().string())
            .col(ColumnDef::new(Column::Temporary).not_null().boolean())
            .col(ColumnDef::new(Column::Ext1Idx).not_null().string())
            .col(ColumnDef::new(Column::Ext2Idx).not_null().string())
            .col(ColumnDef::new(Column::Ext3Idx).not_null().string())
            .col(ColumnDef::new(Column::Ext4).not_null().string())
            .col(ColumnDef::new(Column::Ext5).not_null().string())
            .col(ColumnDef::new(Column::Ext6).not_null().string())
            .col(ColumnDef::new(Column::Ext7).not_null().string())
            .col(ColumnDef::new(Column::Ext8).not_null().string())
            .col(ColumnDef::new(Column::Ext9).not_null().string())
            .col(ColumnDef::new(Column::OwnPaths).not_null().string());
        if db == DatabaseBackend::MySql {
            builder.engine("InnoDB").character_set("utf8mb4").collate("utf8mb4_0900_as_cs");
        }
        builder.to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![
            Index::create().name(&format!("idx-{}-idx1", Entity.table_name())).table(Entity).col(Column::Ext1Idx).to_owned(),
            Index::create().name(&format!("idx-{}-idx2", Entity.table_name())).table(Entity).col(Column::Ext2Idx).to_owned(),
            Index::create().name(&format!("idx-{}-idx3", Entity.table_name())).table(Entity).col(Column::Ext3Idx).to_owned(),
        ]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
