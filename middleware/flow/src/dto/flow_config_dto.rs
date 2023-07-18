use serde::{Deserialize, Serialize};
use tardis::{web::poem_openapi,db::sea_orm, chrono::{DateTime, Utc}};

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowConfigEditReq {
    pub code: String,
    pub value: String,
}