use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use tardis::TardisCreateIndex;

/// Credential or authentication instance model
///
/// Uniform use of cert refers to credentials or authentication
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateIndex)]
#[sea_orm(table_name = "rbum_cert")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub kind: String,
    pub supplier: String,
    /// Cert name \
    /// E.g. username, phone number, app id
    pub ak: String,
    /// Cert key \
    /// E.g. password, token, secret key
    pub sk: String,
    /// Whether the key is visible \
    pub sk_invisible: bool,
    /// Extend information \
    /// The content and format are set by the upper service itself
    pub ext: String,
    /// Specifies the start time for the effective date
    pub start_time: chrono::DateTime<Utc>,
    /// Specifies the end time for the effective date
    pub end_time: chrono::DateTime<Utc>,
    /// Specifies the connection address, mostly for two-party or third-party configurations \
    /// Information from cert config can be overridden
    /// E.g. http://127.0.0.1:8080/api/v1/
    pub conn_uri: String,
    /// @see [status](crate::rbum::rbum_enumeration::RbumCertStatusKind)
    pub status: i16,
    /// Associated [cert configuration](crate::rbum::domain::rbum_cert_conf::Model) id
    pub rel_rbum_cert_conf_id: String,
    /// Associated [resource kind](crate::rbum::rbum_enumeration::RbumCertRelKind) id
    #[index(index_id = "id")]
    pub rel_rbum_kind: i16,
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
    #[index(index_id = "id")]
    pub rel_rbum_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: chrono::DateTime<Utc>,
    pub update_time: chrono::DateTime<Utc>,
    pub create_by: String,
    pub update_by: String,
}

impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(ctx.own_paths.to_string());
            self.owner = Set(ctx.owner.to_string());
            self.create_by = Set(ctx.owner.to_string());
        }
        self.update_by = Set(ctx.owner.to_string());
    }

    fn create_table_statement(db: DbBackend) -> TableCreateStatement {
        let mut builder = Table::create();
        builder
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            // Specific
            .col(ColumnDef::new(Column::Kind).not_null().string())
            .col(ColumnDef::new(Column::Supplier).not_null().string())
            .col(ColumnDef::new(Column::Ak).not_null().string())
            .col(ColumnDef::new(Column::Sk).not_null().string())
            .col(ColumnDef::new(Column::SkInvisible).not_null().boolean().default(false))
            .col(ColumnDef::new(Column::Ext).not_null().string())
            .col(ColumnDef::new(Column::ConnUri).not_null().string())
            .col(ColumnDef::new(Column::RelRbumCertConfId).not_null().string())
            .col(ColumnDef::new(Column::RelRbumKind).not_null().small_integer())
            .col(ColumnDef::new(Column::RelRbumId).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            .col(ColumnDef::new(Column::Status).not_null().small_integer())
            .col(ColumnDef::new(Column::CreateBy).not_null().string())
            .col(ColumnDef::new(Column::UpdateBy).not_null().string());
        if db == DatabaseBackend::Postgres {
            builder
                .col(ColumnDef::new(Column::StartTime).not_null().timestamp_with_time_zone())
                .col(ColumnDef::new(Column::EndTime).not_null().timestamp_with_time_zone())
                .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp_with_time_zone())
                .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp_with_time_zone());
        } else {
            builder
                .engine("InnoDB")
                .character_set("utf8mb4")
                .collate("utf8mb4_0900_as_cs")
                .col(ColumnDef::new(Column::StartTime).not_null().date_time())
                .col(ColumnDef::new(Column::EndTime).not_null().date_time())
                .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp())
                .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).timestamp());
        }
        builder.to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        tardis_create_index_statement()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
