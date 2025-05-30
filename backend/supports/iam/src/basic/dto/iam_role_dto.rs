use std::collections::HashMap;

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

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamRoleAggCopyReq {
    pub copy_role_id: String,
    pub role: IamRoleAddReq,
    pub sync_account: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamRoleAddReq {
    pub id: Option<TrimString>,
    pub code: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    pub kind: Option<IamRoleKind>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<i64>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub extend_role_id: Option<String>,
    pub in_embed: Option<bool>,
    pub in_base: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamRoleAggModifyReq {
    pub role: Option<IamRoleModifyReq>,
    pub res_ids: Option<Vec<String>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct IamRoleModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,

    pub kind: Option<IamRoleKind>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub sort: Option<i64>,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamRoleBoneResp {
    pub id: String,
    pub name: String,
    pub kind: IamRoleKind,
    pub scope_level: RbumScopeLevelKind,
    pub code: String,
    pub icon: String,
    pub in_base: bool,
    pub in_embed: bool,
    pub extend_role_id: String,
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
    pub sort: i64,
    pub in_base: bool,
    pub in_embed: bool,
    pub extend_role_id: String,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug, Clone)]
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
    pub sort: i64,
    pub in_base: bool,
    pub in_embed: bool,
    pub extend_role_id: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamRoleRelAccountCertResp {
    pub account_id: String,
    pub certs: HashMap<String, String>,
}
