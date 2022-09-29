use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm;
use tardis::web::poem_openapi;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;

use crate::iam_enumeration::IamRoleKind;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamRoleAggAddReq {
    pub role: IamRoleAddReq,
    pub res_ids: Option<Vec<String>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamRoleAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    pub kind: Option<IamRoleKind>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub icon: Option<String>,
    pub sort: Option<u32>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamRoleAggModifyReq {
    pub role: IamRoleModifyReq,
    pub res_ids: Option<Vec<String>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamRoleModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,

    pub kind: Option<IamRoleKind>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub icon: Option<String>,
    pub sort: Option<u32>,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamRoleBoneResp {
    pub id: String,
    pub name: String,
    pub icon: String,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamRoleSummaryResp {
    pub id: String,
    pub name: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub kind: IamRoleKind,
    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,

    pub icon: String,
    pub code: String,
    pub sort: u32,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamRoleDetailResp {
    pub id: String,
    pub name: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub kind: IamRoleKind,
    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,

    pub icon: String,
    pub code: String,
    pub sort: u32,
}
