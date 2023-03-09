use crate::basic::serv::iam_cert_ldap_serv::{AccountFieldMap, OrgFieldMap};
use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamCertConfUserPwdAddOrModifyReq {
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub ak_rule_len_min: u8,
    pub ak_rule_len_max: u8,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub sk_rule_len_min: u8,
    pub sk_rule_len_max: u8,
    pub sk_rule_need_num: bool,
    pub sk_rule_need_uppercase: bool,
    pub sk_rule_need_lowercase: bool,
    pub sk_rule_need_spec_char: bool,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub sk_lock_cycle_sec: i32,
    pub sk_lock_err_times: i16,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub sk_lock_duration_sec: i32,
    pub repeatable: bool,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub expire_sec: i64,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamCertConfUserPwdResp {
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub ak_rule_len_min: u8,
    pub ak_rule_len_max: u8,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub sk_rule_len_min: u8,
    pub sk_rule_len_max: u8,
    pub sk_rule_need_num: bool,
    pub sk_rule_need_uppercase: bool,
    pub sk_rule_need_lowercase: bool,
    pub sk_rule_need_spec_char: bool,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub sk_lock_cycle_sec: i32,
    pub sk_lock_err_times: i16,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub sk_lock_duration_sec: i32,
    pub repeatable: bool,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub expire_sec: i64,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamCertConfMailVCodeAddOrModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_rule: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamCertConfPhoneVCodeAddOrModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_rule: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertConfTokenAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    pub coexist_num: i16,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub expire_sec: Option<i64>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertConfTokenModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    pub coexist_num: Option<i16>,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub expire_sec: Option<i64>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamCertConfOAuth2AddOrModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub supplier: TrimString,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub sk: TrimString,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamCertConfOAuth2Resp {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak: String,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub sk: String,
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertConfAkSkAddOrModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub expire_sec: Option<i64>,
}
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug,Clone)]
pub struct IamCertConfLdapAddOrModifyReq {
    /// Assign a code to the LdapCertConf,Used to distinguish different sources
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub supplier: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: String,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub conn_uri: String,
    pub is_tls: bool,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub principal: TrimString,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub credentials: TrimString,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub base_dn: String,
    pub enabled: bool,

    pub port: Option<u16>,
    pub account_unique_id: String,
    pub account_field_map: AccountFieldMap,

    pub org_unique_id: String,
    pub org_field_map: OrgFieldMap,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamCertConfLdapResp {
    pub supplier: String,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub conn_uri: String,
    pub is_tls: bool,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub principal: String,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub credentials: String,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub base_dn: String,
    pub port: u16,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub account_unique_id: String,
    pub account_field_map: AccountFieldMap,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub org_unique_id: String,
    pub org_field_map: OrgFieldMap,
}

impl IamCertConfLdapResp {
    //模糊搜索账号语句
    pub fn package_filter_by_fuzzy_search_account(&self, user_or_display_name: &str) -> String {
        if let Some(search_base_filter) = self.account_field_map.search_base_filter.clone() {
            format!(
                "(&({})(|({}=*{}*)({}=*{}*)))",
                search_base_filter, self.account_unique_id, user_or_display_name, self.account_field_map.field_display_name, user_or_display_name
            )
        } else {
            // such as `(|(cn=*test*)(displayName=*test*))`
            format!(
                "(|({}=*{}*)({}=*{}*))",
                self.account_unique_id, user_or_display_name, self.account_field_map.field_display_name, user_or_display_name
            )
        }
    }
    //根据唯一标识精确搜索
    pub fn package_filter_by_accurate_search(&self, user_or_display_name: &str) -> String {
        if let Some(search_base_filter) = self.account_field_map.search_base_filter.clone() {
            format!("(&({})({}={}))", search_base_filter, self.account_unique_id, user_or_display_name)
        } else {
            format!("{}={}", self.account_unique_id, user_or_display_name)
        }
    }
}
