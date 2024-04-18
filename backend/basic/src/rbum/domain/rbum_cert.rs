use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use tardis::TardisCreateIndex;

/// Certification model
///
/// 凭证模型
///
///
/// This model is an instantiation model of [`crate::rbum::domain::rbum_cert_conf::Model`].
///
/// 此模型是[`crate::rbum::domain::rbum_cert_conf::Model`]的实例化模型。
///
/// NOTE: If you do not need to perform unified verification processing on the credentials,
/// you can use this model directly without associating the credential configuration.
/// For example, data connection credentials, depending on business requirements, may not require credential configuration.
///
/// NOTE: 如果不需要对凭证作统一的校验处理，可以直接使用此模型，不用关联凭证配置。比如数据连接凭证，视业务需求也可以不需要凭证配置。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateIndex)]
#[sea_orm(table_name = "rbum_cert")]
pub struct Model {
    /// Certification id
    ///
    /// 凭证id
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Certification access key
    ///
    /// 凭证名
    ///
    /// see [`crate::rbum::domain::rbum_cert_conf::Model::ak_rule`]
    pub ak: String,
    /// Certification secret key
    ///
    /// 凭证密钥
    ///
    /// see [`crate::rbum::domain::rbum_cert_conf::Model::sk_rule`]
    pub sk: String,
    /// Whether to hide the sk
    ///
    /// 是否隐藏密钥
    ///
    /// In some scenarios with high security requirements, you can choose to hide the key, such as: display as "******".
    ///
    /// 在一些安全性要求较高的场景下，可以选择隐藏密钥，如：显示为“******”。
    pub sk_invisible: bool,
    /// Certificate type
    ///
    /// 凭证类型
    ///
    /// Different from [`crate::rbum::domain::rbum_cert_conf::Model::kind`], when this data exists, it indicates that the certificate does not need to be associated with the certificate configuration.
    ///
    /// 与 [`crate::rbum::domain::rbum_cert_conf::Model::kind`] 不同，当存在此数据时表明该凭证不用关联凭证配置。
    pub kind: String,
    /// Certificate supplier
    ///
    /// 凭证供应商
    ///
    /// Different from [`crate::rbum::domain::rbum_cert_conf::Model::supplier`], when this data exists, it indicates that the certificate does not need to be associated with the certificate configuration.
    ///
    /// 与 [`crate::rbum::domain::rbum_cert_conf::Model::supplier`] 不同，当存在此数据时表明该凭证不用关联凭证配置。
    pub supplier: String,
    /// Certificate extension information
    ///
    /// 凭证扩展信息
    ///
    /// Such as database connection pool configuration.
    ///
    /// 比如数据库连接池配置。
    ///
    /// Different from [`crate::rbum::domain::rbum_cert_conf::Model::ext`], this field is used to identify the specific extension information of the certificate.
    ///
    /// 与 [`crate::rbum::domain::rbum_cert_conf::Model::ext`] 不同，此字段用于标识该条凭证的特有的扩展信息。
    pub ext: String,
    /// Certificate effective time
    ///
    /// 凭证的生效时间
    pub start_time: chrono::DateTime<Utc>,
    /// Certificate expiration time
    ///
    /// 凭证的失效时间
    pub end_time: chrono::DateTime<Utc>,
    /// Certificate connection address
    ///
    /// 凭证连接地址
    ///
    /// Different from [`crate::rbum::domain::rbum_cert_conf::Model::conn_uri`], this field is used to identify the specific connection address of the certificate.
    ///
    /// 与 [`crate::rbum::domain::rbum_cert_conf::Model::conn_uri`] 不同，此字段用于标识该条凭证的特有的连接地址。
    pub conn_uri: String,
    /// Credential status
    ///
    /// 凭证的状态
    ///
    /// see [`crate::rbum::rbum_enumeration::RbumCertConfStatusKind`]
    pub status: i16,
    /// Associated [cert configuration](crate::rbum::domain::rbum_cert_conf::Model) id
    ///
    /// 关联的[凭证配置](crate::rbum::domain::rbum_cert_conf::Model)id
    pub rel_rbum_cert_conf_id: String,
    /// Associated [resource kind](crate::rbum::rbum_enumeration::RbumCertRelKind) id
    ///
    /// 关联的[资源类型](crate::rbum::rbum_enumeration::RbumCertRelKind)id
    #[index(index_id = "id")]
    pub rel_rbum_kind: i16,
    /// Associated resource id
    ///
    /// 关联的资源id
    ///
    /// # examples:
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
    ///
    /// # 使用示例：
    ///
    /// * 如果 rel_rbum_kind == Item
    ///  - rel_rbum_id 与 cert configuration 的 rel_rbum_item_id 相同：比如 Gitlab token
    ///  - rel_rbum_id 与 cert configuration 的 rel_rbum_item_id 不同：比如 用户密码（cert configuration 绑定租户，cert 实例对应用户）
    ///
    /// * 如果 rel_rbum_kind == Set
    /// - 比如在插件服务中，可以绑定到插件实例库
    ///
    /// * 如果 rel_rbum_kind == Rel
    /// - 在 CMDB 服务中，一个资源可以被切片（比如 DB 实例），我们可以指定关联的切片信息
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
