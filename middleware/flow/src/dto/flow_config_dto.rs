use serde::{Deserialize, Serialize};
use tardis::{web::poem_openapi,db::sea_orm, chrono::{DateTime, Utc}};

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowConfigAddReq {
    pub code: String,
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowConfigModifyReq {
    pub code: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowConfigSummaryResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub value: String,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}