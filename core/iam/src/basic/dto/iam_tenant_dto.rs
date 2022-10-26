use crate::basic::dto::iam_cert_conf_dto::{IamCertConfLdapAddOrModifyReq, IamCertConfUserPwdResp};
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm;
use tardis::web::poem_openapi;

use super::iam_cert_conf_dto::{IamCertConfOAuth2AddOrModifyReq, IamCertConfOAuth2Resp, IamCertConfUserPwdAddOrModifyReq};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamTenantAddReq {
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
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    pub account_self_reg: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamTenantModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub icon: Option<String>,
    pub sort: Option<u32>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub contact_phone: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    pub account_self_reg: Option<bool>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamTenantAggAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub icon: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub contact_phone: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    pub account_self_reg: Option<bool>,
    pub disabled: Option<bool>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub admin_username: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub admin_password: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub admin_name: TrimString,

    pub cert_conf_by_user_pwd: IamCertConfUserPwdAddOrModifyReq,
    pub cert_conf_by_phone_vcode: bool,
    pub cert_conf_by_mail_vcode: bool,
    pub cert_conf_by_wechat_mp: Option<IamCertConfOAuth2AddOrModifyReq>,
    #[deprecated = "remove tenant ldap login in conf"]
    pub cert_conf_by_ldap: Option<Vec<IamCertConfLdapAddOrModifyReq>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamTenantAggModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub icon: Option<String>,
    pub sort: Option<u32>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub contact_phone: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub note: Option<String>,
    pub account_self_reg: Option<bool>,
    pub disabled: Option<bool>,

    pub cert_conf_by_user_pwd: Option<IamCertConfUserPwdAddOrModifyReq>,
    pub cert_conf_by_phone_vcode: Option<bool>,
    pub cert_conf_by_mail_vcode: Option<bool>,
    pub cert_conf_by_wechat_mp: Option<IamCertConfOAuth2AddOrModifyReq>,
    #[deprecated = "remove tenant ldap login in conf"]
    pub cert_conf_by_ldap: Option<Vec<IamCertConfLdapAddOrModifyReq>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamTenantAggDetailResp {
    pub id: String,
    pub name: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub disabled: bool,

    pub icon: String,
    pub sort: u32,
    pub contact_phone: String,
    pub note: String,
    pub account_self_reg: bool,

    pub cert_conf_by_user_pwd: IamCertConfUserPwdResp,
    pub cert_conf_by_phone_vcode: bool,
    pub cert_conf_by_mail_vcode: bool,
    pub cert_conf_by_wechat_mp: Option<IamCertConfOAuth2Resp>,
    pub cert_conf_by_ldap: Option<Vec<IamCertConfLdapAddOrModifyReq>>,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamTenantBoneResp {
    pub id: String,
    pub name: String,
    pub icon: String,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamTenantSummaryResp {
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
    pub note: String,
    pub account_self_reg: bool,
}

#[derive(poem_openapi::Object, sea_orm::FromQueryResult, Serialize, Deserialize, Debug)]
pub struct IamTenantDetailResp {
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
    pub sort: u32,
    pub contact_phone: String,
    pub note: String,
    pub account_self_reg: bool,
}
