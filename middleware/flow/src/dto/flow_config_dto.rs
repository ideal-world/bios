use serde::{Deserialize, Serialize};
use tardis::{
    chrono::{DateTime, Utc},
    db::sea_orm,
    web::poem_openapi,
};

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowConfigEditReq {
    pub code: String,
    pub value: String,
}
