use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

/// Credential or authentication instance model
///
/// Uniform use of cert refers to credentials or authentication
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rbum_cert")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Cert name \
    /// E.g. username, phone number, app id
    pub ak: String,
    /// Cert key \
    /// E.g. password, token, secret key
    pub sk: String,
    /// Extend information \
    /// The content and format are set by the upper service itself
    pub ext: String,
    /// Specifies the start time for the effective date
    pub start_time: DateTime,
    /// Specifies the end time for the effective date
    pub end_time: DateTime,
    /// Specifies the connection address, mostly for two-party or third-party configurations \
    /// Information from cert config can be overridden
    /// E.g. http://localhost:8080/api/v1/
    pub conn_uri: String,
    /// @see [status](crate::rbum::rbum_enumeration::RbumCertStatusKind)
    pub status: u8,
    /// Associated [cert configuration](crate::rbum::domain::rbum_cert_conf::Model) id
    pub rel_rbum_cert_conf_id: String,
    /// Associated [resource kind](crate::rbum::rbum_enumeration::RbumCertRelKind) id
    pub rel_rbum_kind: u8,
    /// Associated resource id
    ///
    /// Usage examples:
    ///
    /// * if rel_rbum_kind == Item
    ///   - rel_rbum_id same as the rel_rbum_item_id of cert configuration：E.g. Gitlab token
    ///   - rel_rbum_id different as the rel_rbum_item_id of cert configuration：E.g. User password (the cert configuration is bound to the tenant, and the cert instance corresponds to the user)
    ///
    /// * if rel_rbum_kind == Set
    ///   - E.g. In the Plug-in service, it can be bound to the plug-in instance library
    ///
    /// * if rel_rbum_kind == Rel
    ///  - In the CMDB service, a resource can be sliced (E.g. DB instance), we can specify slice information of association
    pub rel_rbum_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime,
    pub update_time: DateTime,
}

impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(ctx.own_paths.to_string());
            self.owner = Set(ctx.owner.to_string());
        }
    }

    fn create_table_statement(_: DbBackend) -> TableCreateStatement {
        Table::create()
            .table(Entity.table_ref())
            .if_not_exists()
            .engine("InnoDB")
            .character_set("utf8mb4")
            .collate("utf8mb4_0900_as_cs")
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            // Specific
            .col(ColumnDef::new(Column::Ak).not_null().string())
            .col(ColumnDef::new(Column::Sk).not_null().string())
            .col(ColumnDef::new(Column::Ext).not_null().string())
            .col(ColumnDef::new(Column::StartTime).not_null().date_time())
            .col(ColumnDef::new(Column::EndTime).not_null().date_time())
            .col(ColumnDef::new(Column::ConnUri).not_null().string())
            .col(ColumnDef::new(Column::Status).not_null().tiny_unsigned())
            .col(ColumnDef::new(Column::RelRbumCertConfId).not_null().string())
            .col(ColumnDef::new(Column::RelRbumKind).not_null().tiny_unsigned())
            .col(ColumnDef::new(Column::RelRbumId).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
            .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).date_time())
            .to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![
            Index::create().name(&format!("idx-{}-ak", Entity.table_name())).table(Entity).col(Column::OwnPaths).col(Column::RelRbumKind).col(Column::Ak).unique().to_owned(),
            Index::create()
                .name(&format!("idx-{}-{}", Entity.table_name(), Column::RelRbumKind.to_string()))
                .table(Entity)
                .col(Column::RelRbumKind)
                .col(Column::RelRbumId)
                .to_owned(),
        ]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
