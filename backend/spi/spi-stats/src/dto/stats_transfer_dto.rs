use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    serde_json::Value,
    web::poem_openapi,
};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsExportDataReq {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsExportDataResp {
    pub fact_conf_data: HashMap<String, Vec<StatsExportAggResp>>,
    pub fact_conf_data_del: HashMap<String, Vec<StatsExportDelAggResp>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct StatsImportDataReq {
    pub fact_conf_data: HashMap<String, Vec<StatsImportAggReq>>,
    pub fact_conf_data_del: HashMap<String, Vec<StatsImportDelAggReq>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsExportAggResp {
    /// Primary key
    pub key: String,
    /// Own paths
    pub own_paths: String,
    /// Create time
    pub ct: DateTime<Utc>,
    /// Idempotent id
    /// ps: The idempotent id is used to ensure that the same request is not processed repeatedly
    ///
    /// 幂等id
    /// ps: 幂等id用于确保同一个请求不会重复处理
    pub idempotent_id: Option<String>,
    /// Field data
    /// 字段数据
    ///
    /// Map format，key = field name of the fact table，value = field value
    /// Map格式，key=事实表的字段名，value=字段值
    pub data: Value,

    /// Dynamic data
    ///
    /// 动态数据
    pub ext: Option<Value>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct StatsExportDelAggResp {
    /// Primary key
    pub key: String,
    /// Create time
    pub ct: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct StatsImportAggReq {
    /// Primary key
    pub key: String,
    /// Own paths
    pub own_paths: String,
    /// Create time
    pub ct: DateTime<Utc>,
    /// Idempotent id
    /// ps: The idempotent id is used to ensure that the same request is not processed repeatedly
    ///
    /// 幂等id
    /// ps: 幂等id用于确保同一个请求不会重复处理
    pub idempotent_id: Option<String>,
    /// Field data
    /// 字段数据
    ///
    /// Map format，key = field name of the fact table，value = field value
    /// Map格式，key=事实表的字段名，value=字段值
    pub data: Value,

    /// Dynamic data
    ///
    /// 动态数据
    pub ext: Option<Value>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct StatsImportDelAggReq {
    /// Primary key
    pub key: String,
    /// Create time
    pub ct: DateTime<Utc>,
}
