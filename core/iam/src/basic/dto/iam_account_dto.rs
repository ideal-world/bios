use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm;
use tardis::web::poem_openapi;

use bios_basic::rbum::rbum_enumeration::{RbumCertStatusKind, RbumScopeLevelKind};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
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
    pub org_node_ids: Option<Vec<String>>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    pub exts: HashMap<String, String>,
    pub status: Option<RbumCertStatusKind>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
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

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAccountAggModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,

    pub role_ids: Option<Vec<String>>,
    pub org_cate_ids: Option<Vec<String>>,

    pub exts: Option<HashMap<String, String>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAccountModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAccountSelfModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,

    pub exts: HashMap<String, String>,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamAccountBoneResp {
    pub id: String,
    pub name: String,
    pub icon: String,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
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

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamAccountDetailResp {
    pub id: String,
    pub name: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,

    pub icon: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAccountSummaryAggResp {
    pub id: String,
    pub name: String,

    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,

    pub icon: String,

    pub roles: HashMap<String, String>,
    pub certs: HashMap<String, String>,
    pub orgs: Vec<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAccountDetailAggResp {
    pub id: String,
    pub name: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,

    pub icon: String,

    pub roles: HashMap<String, String>,
    pub certs: HashMap<String, String>,
    pub orgs: Vec<String>,
    pub exts: Vec<IamAccountAttrResp>,
    pub groups: HashMap<String, String>,
    pub apps: Vec<IamAccountAppInfoResp>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAccountAttrResp {
    pub name: String,
    pub label: String,
    pub value: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAccountInfoResp {
    pub account_id: String,
    pub account_name: String,
    pub token: String,
    pub access_token: Option<String>,
    pub roles: HashMap<String, String>,
    pub groups: HashMap<String, String>,
    pub apps: Vec<IamAccountAppInfoResp>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAccountInfoWithUserPwdAkResp {
    pub iam_account_info_resp: IamAccountInfoResp,
    pub ak: String,
    pub status: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAccountAppInfoResp {
    pub app_id: String,
    pub app_name: String,
    pub roles: HashMap<String, String>,
    pub groups: HashMap<String, String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamAccountExtSysResp {
    pub account_id: String,
    pub user_name: String,
    pub display_name: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamAccountExtSysAddReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub account_id: String,
    pub code: TrimString,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamAccountExtSysBatchAddReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub account_id: Vec<String>,
    pub code: TrimString,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamAccountAddByLdapResp {
    pub result: Vec<String>,
    pub fail: HashMap<String, String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamCpUserPwdBindResp {
    /// true=is bind ,false=not bind
    pub is_bind: bool,
}
