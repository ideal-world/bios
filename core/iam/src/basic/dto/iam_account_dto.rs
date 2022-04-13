use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::FromQueryResult;
use tardis::web::poem_openapi::Object;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(Object, Serialize, Deserialize, Debug)]
#[oai(rename_all = "camelCase")]
pub struct IamAccountAddReq {
    pub id: Option<TrimString>,
    pub name: TrimString,
    pub scope_level: RbumScopeLevelKind,
    pub disabled: Option<bool>,

    pub icon: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
#[oai(rename_all = "camelCase")]
pub struct IamAccountModifyReq {
    pub name: Option<TrimString>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    pub icon: Option<String>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
#[oai(rename_all = "camelCase")]
pub struct IamAccountSummaryResp {
    pub id: String,
    pub name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,

    pub icon: String,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
#[oai(rename_all = "camelCase")]
pub struct IamAccountDetailResp {
    pub id: String,
    pub name: String,

    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,

    pub icon: String,
}
