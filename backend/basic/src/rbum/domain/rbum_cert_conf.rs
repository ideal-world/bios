use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Credential or authentication configuration model
///
/// Uniform use of cert refers to credentials or authentication
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_cert_conf")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[tardis_entity(custom_len = "127")]
    #[index(index_id = "id_2", unique)]
    pub kind: String,
    #[tardis_entity(custom_len = "127")]
    #[index(index_id = "id_2", unique)]
    pub supplier: String,
    pub name: String,
    pub note: String,
    pub ak_note: String,
    pub ak_rule: String,
    pub sk_note: String,
    pub sk_rule: String,
    #[tardis_entity(custom_type = "text")]
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
