use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::FromQueryResult;
use tardis::web::poem_openapi::Object;

#[derive(Serialize, Deserialize, Debug)]
pub struct IamRoleAddReq {
    pub name: TrimString,
    pub icon: Option<String>,
    pub sort: Option<u32>,

    pub scope_level: i32,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IamRoleModifyReq {
    pub name: Option<TrimString>,
    pub icon: Option<String>,
    pub sort: Option<u32>,

    pub scope_level: Option<i32>,
    pub disabled: Option<bool>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamRoleSummaryResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub sort: u32,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: i32,
    pub disabled: bool,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamRoleDetailResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub sort: u32,

    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: i32,
    pub disabled: bool,
}
