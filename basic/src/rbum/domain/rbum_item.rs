use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

/// Resource model
///
/// Used to represent a specific resource,
/// Each resource corresponds to a [resource kind](crate::rbum::domain::rbum_kind::Model)  and [resource domain](crate::rbum::domain::rbum_domain::Model).
///
/// Each resource corresponds to a unique uri,
/// and the uri consists of `<resource kind>://<resource domain>/<resource code>`
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_item")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Resource code
    pub code: String,
    pub name: String,
    /// Associated [resource kind](crate::rbum::domain::rbum_kind::Model) id
    pub rel_rbum_kind_id: String,
    /// Associated [resource domain](crate::rbum::domain::rbum_domain::Model) id
    pub rel_rbum_domain_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: chrono::DateTime<Utc>,
    pub update_time: chrono::DateTime<Utc>,

    pub scope_level: i16,

    pub disabled: bool,
}

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
            .col(ColumnDef::new(Column::Code).not_null().string())
            .col(ColumnDef::new(Column::Name).not_null().string())
            .col(ColumnDef::new(Column::RelRbumKindId).not_null().string())
            .col(ColumnDef::new(Column::RelRbumDomainId).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            // With Scope
            .col(ColumnDef::new(Column::ScopeLevel).not_null().small_integer())
            // With Status
            .col(ColumnDef::new(Column::Disabled).not_null().boolean());
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

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::OwnPaths.to_string())).table(Entity).col(Column::OwnPaths).to_owned(),
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::RelRbumKindId.to_string())).table(Entity).col(Column::RelRbumKindId).to_owned(),
            Index::create().name(&format!("idx-{}-{}", Entity.table_name(), Column::RelRbumDomainId.to_string())).table(Entity).col(Column::RelRbumDomainId).to_owned(),
            Index::create()
                .name(&format!("idx-{}-{}", Entity.table_name(), Column::Code.to_string()))
                .table(Entity)
                .col(Column::Code)
                .col(Column::RelRbumKindId)
                .col(Column::RelRbumDomainId)
                .unique()
                .to_owned(),
        ]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
