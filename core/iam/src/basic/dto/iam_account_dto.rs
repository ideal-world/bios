use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::FromQueryResult;
use tardis::web::poem_openapi::Object;

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamAccountAggAddReq {
    #[oai(skip = true)]
    pub id: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub cert_user_name: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub cert_password: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub cert_phone: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub cert_mail: Option<TrimString>,

    pub role_ids: Option<Vec<String>>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub exts: HashMap<String, String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamAccountAddReq {
    #[oai(skip = true)]
    pub id: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamAccountAggModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,

    pub role_ids: Option<Vec<String>>,

    pub exts: HashMap<String, String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamAccountModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamAccountSelfModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,

    pub exts: HashMap<String, String>,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamAccountSummaryResp {
    pub id: String,
    pub name: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,

    pub icon: String,
}

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamAccountDetailResp {
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
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamAccountDetailWithExtResp {
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
    pub exts: HashMap<String, String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct AccountInfoResp {
    pub account_id: String,
    pub account_name: String,
    pub token: String,
    pub roles: HashMap<String, String>,
    pub groups: HashMap<String, String>,
    pub apps: Vec<AccountAppInfoResp>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct AccountAppInfoResp {
    pub app_id: String,
    pub app_name: String,
    pub roles: HashMap<String, String>,
    pub groups: HashMap<String, String>,
}
