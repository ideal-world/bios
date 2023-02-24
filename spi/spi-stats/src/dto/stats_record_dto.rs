use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    serde_json::Value,
    web::poem_openapi,
};

/// Load Fact Record Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsFactRecordLoadReq {
    pub own_paths: String,
    pub ct: DateTime<Utc>,
    pub data: Value,
}

/// Load Fact Record Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsFactRecordsLoadReq {
    pub key: String,
    pub own_paths: String,
    pub ct: DateTime<Utc>,
    pub data: Value,
}

/// Add Dimension Record Request Object
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsDimRecordAddReq {
    pub show_name: String,
    pub parent_key: Option<String>,
    pub ct: DateTime<Utc>,
}
