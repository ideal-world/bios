use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use tardis::TardisCreateIndex;

/// Credential or authentication configuration model
///
/// Uniform use of cert refers to credentials or authentication
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateIndex)]
#[sea_orm(table_name = "rbum_cert_conf")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[index(index_id = "id_2", unique)]
    pub kind: String,
    #[index(index_id = "id_2", unique)]
    pub supplier: String,
    pub name: String,
    pub note: String,
    pub ak_note: String,
    pub ak_rule: String,
    pub sk_note: String,
    pub sk_rule: String,
    pub ext: String,
    pub sk_need: bool,
    /// Whether dynamic sk \
    /// If true, the sk will be stored in the cache
    pub sk_dynamic: bool,
    pub sk_encrypted: bool,
    /// Whether sk can be repeated \
    /// If true, the sk can be modified to the same sk as the current one when it expires
    pub repeatable: bool,
    /// Whether it is a basic authentication \
    /// There can only be at most one base certification for the same `rel_rbum_item_id` \
    /// If true, the sk of this record will be the public sk of the same `rel_rbum_item_id` ,
    /// supports a login method like ak of different cert configuration in the same `rel_rbum_item_id` + sk of this record
    pub is_basic: bool,
    /// Whether ak can be repeated \
    /// If true, ak can be same in different record
    pub is_ak_repeatable: bool,
    /// Support reset the cert configuration type(corresponding to the 'code' value) of the basic sk \
    /// Multiple values are separated by commas
    pub rest_by_kinds: String,
    /// The expiration time of the Sk
    pub expire_sec: i64,
    pub sk_lock_cycle_sec: i32,
    pub sk_lock_err_times: i16,
    pub sk_lock_duration_sec: i32,
    /// The number of simultaneously valid \
    /// Used to control the number of certs in effect, E.g.
    /// * Single terminal sign-on: configure a record：`code` = 'token' & `coexist_num` = 1
    /// * Can log in to one android, ios, two web terminals at the same time: configure 3 records：
    ///  `code` = 'token_android' & `coexist_num` = 1 , `code` = 'token_ios' & `coexist_num` = 1 , `code` = 'token_web' & `coexist_num` = 2
    pub coexist_num: i16,
    /// Specifies the connection address, mostly for two-party or third-party configurations \
    /// E.g. http://127.0.0.1:8080/api/v1/
    pub conn_uri: String,
    /// see [status][crate::rbum::rbum_enumeration::RbumCertConfStatusKind]
    pub status: i16,
    /// Associated [resource domain](crate::rbum::domain::rbum_domain::Model) id
    #[index(index_id = "id_2", unique)]
    pub rel_rbum_domain_id: String,
    /// Associated [resource](crate::rbum::domain::rbum_item::Model) id
    #[index(index_id = "id_2", unique)]
    pub rel_rbum_item_id: String,

    #[index()]
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
            .col(ColumnDef::new(Column::Kind).not_null().string_len(127))
            .col(ColumnDef::new(Column::Supplier).not_null().string_len(127))
            .col(ColumnDef::new(Column::Name).not_null().string())
            .col(ColumnDef::new(Column::Note).not_null().string())
            .col(ColumnDef::new(Column::AkNote).not_null().string())
            .col(ColumnDef::new(Column::AkRule).not_null().string())
            .col(ColumnDef::new(Column::SkNote).not_null().string())
            .col(ColumnDef::new(Column::SkRule).not_null().string())
            .col(ColumnDef::new(Column::Ext).text())
            .col(ColumnDef::new(Column::SkNeed).not_null().boolean())
            .col(ColumnDef::new(Column::SkDynamic).not_null().boolean())
            .col(ColumnDef::new(Column::SkEncrypted).not_null().boolean())
            .col(ColumnDef::new(Column::Repeatable).not_null().boolean())
            .col(ColumnDef::new(Column::IsBasic).not_null().boolean())
            .col(ColumnDef::new(Column::IsAkRepeatable).not_null().boolean())
            .col(ColumnDef::new(Column::RestByKinds).not_null().string())
            .col(ColumnDef::new(Column::ExpireSec).not_null().big_integer())
            .col(ColumnDef::new(Column::SkLockCycleSec).not_null().integer())
            .col(ColumnDef::new(Column::SkLockErrTimes).not_null().small_integer())
            .col(ColumnDef::new(Column::SkLockDurationSec).not_null().integer())
            .col(ColumnDef::new(Column::CoexistNum).not_null().small_integer())
            .col(ColumnDef::new(Column::ConnUri).not_null().string())
            .col(ColumnDef::new(Column::RelRbumDomainId).not_null().string())
            .col(ColumnDef::new(Column::RelRbumItemId).not_null().string())
            .col(ColumnDef::new(Column::Status).not_null().small_integer())
            // Basic
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            .col(ColumnDef::new(Column::CreateBy).not_null().string())
            .col(ColumnDef::new(Column::UpdateBy).not_null().string());
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
        tardis_create_index_statement()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
