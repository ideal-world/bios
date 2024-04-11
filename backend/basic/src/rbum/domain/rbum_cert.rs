use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Credential or authentication instance model
///
/// Uniform use of cert refers to credentials or authentication
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
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

    #[fill_ctx(fill = "own_paths")]
    pub own_paths: String,
    #[fill_ctx]
    pub owner: String,
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub create_time: chrono::DateTime<Utc>,
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub update_time: chrono::DateTime<Utc>,
    #[fill_ctx]
    pub create_by: String,
    #[fill_ctx(insert_only = false)]
    pub update_by: String,
}
