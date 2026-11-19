use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// 工作流运行级凭证续期授权；绑定 `run_id + secret_key`，capability 只保存 SHA-256 哈希，
/// 明文仅在注册响应中出现一次，由受信宿主保存在运行级密封 secret 信封内。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "iam_token_renewal")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub run_id: String,
    pub secret_key: String,
    pub account_id: String,
    pub own_paths: String,
    /// 注册时用户上下文带应用上下文（`tenant/app`）则记录 app，用于兑换时重新校验应用成员关系。
    pub app_id: Option<String>,
    pub capability_hash: String,
    /// 续期授权的绝对截止时间（RFC3339）；注册时固定，兑换不向后延长。
    pub expires_at: String,
    pub last_refreshed_at: Option<String>,
    pub revoked: bool,
    pub revoked_by: Option<String>,
    pub revoked_time: Option<String>,
    pub create_time: String,
    #[index(index_id = "current_slot", name = "ux_iam_token_renewal_current_slot", unique, if_not_exists = true)]
    pub current_run_id: Option<String>,
    #[index(index_id = "current_slot", unique, if_not_exists = true)]
    pub current_secret_key: Option<String>,
    pub replaced_by_credential_id: Option<String>,
}
