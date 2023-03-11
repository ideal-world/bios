use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    serde_json::Value,
    web::poem_openapi,
};

/// Load Fact Record Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsFactRecordLoadReq {
    /// Own paths
    pub own_paths: String,
    /// Create time
    pub ct: DateTime<Utc>,
    /// Field data
    ///
    /// Map format，key = field name of the fact table，value = field value
    pub data: Value,
}

/// Load Fact Record Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsFactRecordsLoadReq {
    /// Primary key
    pub key: String,
    /// Own paths
    pub own_paths: String,
    /// Create time
    pub ct: DateTime<Utc>,
    /// Field data
    ///
    /// Map format，key = field name of the fact table，value = field value
    pub data: Value,
}

/// Add Dimension Record Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsDimRecordAddReq {
    /// Primary key
    pub key: Value,
    /// The name of the dimension
    pub show_name: String,
    /// The parent primary key, if present, indicates that this is a multilevel record
    pub parent_key: Option<Value>,
}

/// Delete Dimension Record Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsDimRecordDeleteReq {
    /// Primary key
    pub key: Value,
}
