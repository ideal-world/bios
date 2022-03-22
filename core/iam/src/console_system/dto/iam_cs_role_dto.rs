use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::FromQueryResult;
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCsRoleAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCsRoleModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamCsRoleSummaryResp {
    pub name: String,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamCsRoleDetailResp {
    pub id: String,
    pub name: String,

    pub updater_code: String,
    pub updater_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
