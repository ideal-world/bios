use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::FromQueryResult;
use tardis::web::poem_openapi::Object;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(Object, Serialize, Deserialize, Debug)]
#[oai(rename_all = "camelCase")]
pub struct IamTenantAddReq {
    pub name: TrimString,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: Option<bool>,

    pub icon: Option<String>,
    pub sort: Option<u32>,
    pub contact_phone: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
#[oai(rename_all = "camelCase")]
pub struct IamTenantModifyReq {
    pub name: Option<TrimString>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    pub icon: Option<String>,
    pub sort: Option<u32>,
    pub contact_phone: Option<String>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
#[oai(rename_all = "camelCase")]
pub struct IamTenantSummaryResp {
    pub id: String,
    pub name: String,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,

    pub icon: String,
    pub sort: u32,
    pub contact_phone: String,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
#[oai(rename_all = "camelCase")]
pub struct IamTenantDetailResp {
    pub id: String,
    pub name: String,

    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,

    pub icon: String,
    pub sort: u32,
    pub contact_phone: String,
}
