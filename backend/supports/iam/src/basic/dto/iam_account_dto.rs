use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{self, DateTime, Utc};
use tardis::db::sea_orm;
use tardis::web::poem_openapi;

use crate::basic::dto::iam_cert_conf_dto::IamCertConfLdapResp;
use crate::basic::serv::iam_cert_ldap_serv::ldap::LdapSearchResp;
use crate::iam_enumeration::{IamAccountLockStateKind, IamAccountLogoutTypeKind, IamAccountStatusKind};
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
    pub cert_password: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub cert_phone: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub cert_mail: Option<TrimString>,
    pub role_ids: Option<Vec<String>>,
    pub org_node_ids: Option<Vec<String>>,
    pub lock_status: Option<IamAccountLockStateKind>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
    pub logout_type: Option<IamAccountLogoutTypeKind>,

    pub labor_type: Option<String>,

    pub temporary: Option<bool>,

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
    pub logout_type: Option<IamAccountLogoutTypeKind>,
    pub labor_type: Option<String>,
    pub temporary: Option<bool>,
    pub lock_status: Option<IamAccountLockStateKind>,
    pub status: Option<IamAccountStatusKind>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct IamAccountAggModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
    pub logout_type: Option<IamAccountLogoutTypeKind>,
    pub labor_type: Option<String>,
    pub temporary: Option<bool>,
    pub status: Option<IamAccountStatusKind>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub cert_phone: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub cert_mail: Option<TrimString>,

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
    pub logout_type: Option<IamAccountLogoutTypeKind>,
    pub labor_type: Option<String>,
    pub temporary: Option<bool>,
    pub lock_status: Option<IamAccountLockStateKind>,

    pub status: Option<IamAccountStatusKind>,
    pub is_auto: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAccountSelfModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    pub disabled: Option<bool>,
    pub logout_type: Option<IamAccountLogoutTypeKind>,
    pub labor_type: Option<String>,
    // #[oai(validator(min_length = "2", max_length = "1000"))]
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
    pub status: IamAccountStatusKind,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub effective_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
    pub logout_time: chrono::DateTime<Utc>,
    pub logout_type: String,
    pub labor_type: String,

    pub temporary: bool,
    pub lock_status: IamAccountLockStateKind,

    pub icon: String,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamAccountDetailResp {
    pub id: String,
    pub name: String,
    pub status: IamAccountStatusKind,
    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub effective_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
    pub logout_time: chrono::DateTime<Utc>,
    pub logout_type: String,
    pub labor_type: String,

    pub temporary: bool,
    pub lock_status: IamAccountLockStateKind,

    pub icon: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamAccountSummaryAggResp {
    pub id: String,
    pub name: String,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub effective_time: DateTime<Utc>,
    pub logout_time: chrono::DateTime<Utc>,
    pub logout_type: String,
    pub labor_type: String,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
    pub is_locked: bool,
    pub is_online: bool,
    pub status: IamAccountStatusKind,
    pub temporary: bool,
    pub lock_status: IamAccountLockStateKind,
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
    pub effective_time: DateTime<Utc>,
    pub logout_time: chrono::DateTime<Utc>,
    pub logout_type: String,
    pub labor_type: String,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
    pub is_locked: bool,
    pub is_online: bool,
    pub status: IamAccountStatusKind,
    pub temporary: bool,
    pub lock_status: IamAccountLockStateKind,
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
    pub app_icon: String,
    pub roles: HashMap<String, String>,
    pub groups: HashMap<String, String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamAccountExtSysResp {
    pub account_id: String,
    pub user_name: String,
    pub display_name: String,
    pub mobile: String,
    pub email: String,
    pub labor_type: String,
}

impl IamAccountExtSysResp {
    pub fn form_ldap_search_resp(resp: LdapSearchResp, config: &IamCertConfLdapResp) -> IamAccountExtSysResp {
        IamAccountExtSysResp {
            user_name: resp.get_simple_attr(&config.account_field_map.field_user_name).unwrap_or_default(),
            display_name: resp.get_simple_attr(&config.account_field_map.field_display_name).unwrap_or_default(),
            account_id: if config.account_unique_id != "dn" {
                resp.get_simple_attr(&config.account_unique_id).unwrap_or_default()
            } else {
                resp.dn.clone()
            },
            mobile: resp.get_simple_attr(&config.account_field_map.field_mobile).unwrap_or_default(),
            email: resp.get_simple_attr(&config.account_field_map.field_email).unwrap_or_default(),
            labor_type: resp.get_simple_attr(&config.account_field_map.field_labor_type).unwrap_or_default(),
        }
    }
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
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct AccountTenantInfoResp {
    pub tenant_info: HashMap<String, AccountTenantInfo>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Default)]
pub struct AccountTenantInfo {
    pub roles: HashMap<String, String>,
    pub orgs: Vec<String>,
    pub groups: HashMap<String, String>,
    pub apps: Vec<IamAccountAppInfoResp>,
}
