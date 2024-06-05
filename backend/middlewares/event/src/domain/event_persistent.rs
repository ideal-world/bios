
use serde::{Deserialize, Serialize};
use tardis::chrono::Utc;
use tardis::db::sea_orm::{self, DeriveEntityModel, DerivePrimaryKey, DeriveRelation, EntityName, EntityTrait, EnumIter, PrimaryKeyTrait};
use tardis::serde_json::Value;
use tardis::{chrono, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};
/// Event Topic model
///
/// 事件主题模型
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "event_persistent")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub message: Value,
    pub inst_id: String,
    pub mgr_node: bool,
    pub subscribe_mode: bool,
    pub topic: String,
    pub status: String,
    pub error: Option<String>,
    #[sea_orm(extra = "DEFAULT 0")]
    pub retry_times: i32,
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub create_time: chrono::DateTime<Utc>,
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub update_time: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Status {
    Sending,
    Success,
    Failed,
    Unknown,
}

impl Status {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Status::Sending => "Sending",
            Status::Success => "Success",
            Status::Failed => "Failed",
            _ => "Unknown"
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(<&'static str>::from(*self))
    }
}

impl From<Status> for &'static str {
    fn from(val: Status) -> Self {
        val.as_str()
    }
}
impl From<&str> for Status {
    fn from(value: &str) -> Self {
        match value {
            "Sending" => Self::Sending,
            "Success" => Self::Success,
            "Failed" => Self::Failed,
            _ => Status::Unknown,
        }
    }
}