use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// 同一用户、Provider 配置和外部身份共享刷新状态，避免复制 refresh token。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "iam_oauth2_provider_grant")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub account_id: String,
    pub cert_conf_id: String,
    pub external_subject: String,
    #[tardis_entity(custom_type = "Text")]
    pub encrypted_token: String,
    pub update_time: String,
}
