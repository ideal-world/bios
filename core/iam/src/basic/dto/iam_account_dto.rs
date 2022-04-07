use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::FromQueryResult;
use tardis::web::poem_openapi::Object;

#[derive(Serialize, Deserialize, Debug)]
pub struct IamAccountAddReq {
    pub id: Option<TrimString>,
    pub name: TrimString,
    pub icon: Option<String>,

    pub scope_level: i32,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IamAccountModifyReq {
    pub name: Option<TrimString>,
    pub icon: Option<String>,

    pub scope_level: Option<i32>,
    pub disabled: Option<bool>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamAccountSummaryResp {
    pub id: String,
    pub name: String,
    pub icon: String,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: i32,
    pub disabled: bool,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamAccountDetailResp {
    pub id: String,
    pub name: String,
    pub icon: String,

    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: i32,
    pub disabled: bool,
}
