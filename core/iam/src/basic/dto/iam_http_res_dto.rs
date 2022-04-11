use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::FromQueryResult;
use tardis::web::poem_openapi::Object;

#[derive(Serialize, Deserialize, Debug)]
pub struct IamHttpResAddReq {
    pub name: TrimString,
    pub code: TrimString,
    pub icon: Option<String>,
    pub sort: Option<u32>,

    pub method: TrimString,

    pub scope_level: i32,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IamHttpResModifyReq {
    pub name: Option<TrimString>,
    pub code: Option<TrimString>,
    pub icon: Option<String>,
    pub sort: Option<u32>,

    pub method: Option<TrimString>,

    pub scope_level: Option<i32>,
    pub disabled: Option<bool>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamHttpResSummaryResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub icon: String,
    pub sort: u32,

    pub method: String,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: i32,
    pub disabled: bool,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamHttpResDetailResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub icon: String,
    pub sort: u32,

    pub method: String,

    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: i32,
    pub disabled: bool,
}
