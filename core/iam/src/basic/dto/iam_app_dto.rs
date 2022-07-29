use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm;
use tardis::web::poem_openapi;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAppAggAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub app_name: TrimString,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub app_icon: Option<String>,
    pub app_sort: Option<u32>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub app_contact_phone: Option<String>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub admin_id: String,

    pub disabled: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAppAddReq {
    #[oai(skip = true)]
    pub id: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub icon: Option<String>,
    pub sort: Option<u32>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub contact_phone: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAppModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub icon: Option<String>,
    pub sort: Option<u32>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub contact_phone: Option<String>,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamAppSummaryResp {
    pub id: String,
    pub name: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,

    pub icon: String,
    pub sort: u32,
    pub contact_phone: String,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamAppDetailResp {
    pub id: String,
    pub name: String,

    pub own_paths: String,
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
