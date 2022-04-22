use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::FromQueryResult;
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamAccountAddReq {
    pub id: Option<TrimString>,
    pub name: TrimString,
    pub scope_level: RbumScopeLevelKind,
    pub disabled: Option<bool>,

    pub icon: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamAccountModifyReq {
    pub name: Option<TrimString>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    pub icon: Option<String>,
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
