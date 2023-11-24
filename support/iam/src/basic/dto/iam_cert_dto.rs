use std::collections::HashSet;

use crate::iam_enumeration::{IamCertExtKind, WayToAdd, WayToDelete};
use bios_basic::rbum::rbum_enumeration::RbumCertStatusKind;
use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamContextFetchReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub token: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub app_id: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertUserNameNewReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub original_ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub new_ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sk: TrimString,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertPwdNewReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub original_sk: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub new_sk: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertUserPwdAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sk: TrimString,
    pub is_ignore_check_sk: bool,
    pub status: Option<RbumCertStatusKind>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertUserPwdModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub original_sk: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub new_sk: TrimString,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertUserPwdRestReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub new_sk: Option<TrimString>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertGenericValidateSkReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sk: TrimString,
    // when ldap validate , the validate_type is supplier
    pub validate_type: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertMailVCodeAddReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertMailVCodeModifyReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertMailVCodeResendActivationReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertMailVCodeActivateReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub vcode: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertPhoneVCodeAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub phone: TrimString,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertPhoneVCodeModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub phone: TrimString,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertPhoneVCodeBindReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub phone: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub vcode: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamThirdPartyCertExtAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: String,
    #[oai(validator(min_length = "1", max_length = "255"))]
    // todo change to String
    pub supplier: Option<String>,
    #[oai(validator(min_length = "2", max_length = "10000"))]
    pub sk: Option<String>,
    pub ext: Option<String>,
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamThirdIntegrationSyncAddReq {
    pub account_sync_from: IamCertExtKind,
    pub account_sync_cron: Option<String>,
    pub account_way_to_add: Option<WayToAdd>,
    pub account_way_to_delete: Option<WayToDelete>,
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamThirdIntegrationConfigDto {
    pub account_sync_from: IamCertExtKind,
    /// None表示手动
    pub account_sync_cron: Option<String>,
    pub account_way_to_add: WayToAdd,
    pub account_way_to_delete: WayToDelete,
    // pub org_sync_from: IamCertExtKind,
    // pub org_sync_cron: String,
    //
    // ///组织关联账号
    // pub org_rel_account: bool,
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamThirdIntegrationSyncStatusDto {
    pub total: usize,
    pub success: u64,
    pub failed: u64,
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertManageAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: String,
    #[oai(validator(min_length = "2", max_length = "10000"))]
    pub sk: Option<String>,
    #[oai(default)]
    pub sk_invisible: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub conn_uri: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub supplier: String,
    #[oai(validator(min_length = "2", max_length = "10000"))]
    pub ext: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertManageModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: String,
    #[oai(validator(min_length = "2", max_length = "10000"))]
    pub sk: Option<String>,
    #[oai(default)]
    pub sk_invisible: Option<bool>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub conn_uri: Option<String>,
    #[oai(validator(min_length = "2", max_length = "10000"))]
    pub ext: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertOAuth2AddOrModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub open_id: TrimString,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertLdapAddOrModifyReq {
    // ldap account unique id
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ldap_id: TrimString,
    pub status: RbumCertStatusKind,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertAkSkAddReq {
    pub tenant_id: String,
    pub app_id: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertAkSkResp {
    pub id: String,
    pub ak: String,
    pub sk: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamOauth2AkSkResp {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: String,
    pub refresh_token: String,
    pub scope: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertDecodeRequest {
    pub codes: HashSet<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertModifyVisibilityRequest {
    pub sk_invisible: bool,
}
