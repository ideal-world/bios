use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use serde::{Deserialize, Serialize};

use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm;
use tardis::web::poem_openapi;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct IamSubDeployAddReq {
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub province: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub access_url: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub note: Option<String>,
    pub scope_level: Option<RbumScopeLevelKind>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct IamSubDeployModifyReq {
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub province: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub access_url: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub note: Option<String>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct IamSubDeploySummaryResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub province: String,
    pub access_url: String,
    pub note: String,
    pub disabled: bool,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub create_by: String,
    pub update_by: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct IamSubDeployDetailResp {
    pub id: String,
    pub code: String,
    pub name: String,
    pub province: String,
    pub access_url: String,
    pub note: String,
    pub disabled: bool,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub create_by: String,
    pub update_by: String,
}
