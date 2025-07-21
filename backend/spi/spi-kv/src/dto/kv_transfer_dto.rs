use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    db::sea_orm,
    serde_json::Value,
    web::poem_openapi,
};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvExportDataReq {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct KvExportDataResp {
    pub kv_data: Vec<KvExportAggResp>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct KvImportDataReq {
    pub kv_data: Vec<KvImportAggReq>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, sea_orm::FromQueryResult)]
pub struct KvExportAggResp {
    pub key: String,
    pub value: Value,
    pub info: String,
    pub owner: String,
    pub own_paths: String,
    pub disable: bool,
    pub scope_level: i16,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct KvImportAggReq {
    pub key: String,
    pub value: Value,
    pub info: String,
    pub owner: String,
    pub own_paths: String,
    pub disable: bool,
    pub scope_level: i16,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
