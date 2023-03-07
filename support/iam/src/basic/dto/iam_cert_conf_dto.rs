use crate::basic::serv::iam_cert_ldap_serv::{AccountFieldMap, OrgFieldMap};
use crate::iam_config::LdapClientConfig;
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
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
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

impl From<LdapClientConfig> for IamCertConfLdapAddOrModifyReq {
    fn from(iam_ldap_conf: LdapClientConfig) -> Self {
        IamCertConfLdapAddOrModifyReq {
            supplier: iam_ldap_conf.code,
            name: iam_ldap_conf.name,
            conn_uri: iam_ldap_conf.conn_uri,
            is_tls: iam_ldap_conf.is_tls,
            principal: iam_ldap_conf.principal,
            credentials: iam_ldap_conf.credentials,
            base_dn: iam_ldap_conf.base_dn,
            field_display_name: iam_ldap_conf.field_display_name,
            search_base_filter: iam_ldap_conf.search_base_filter,
            enabled: true,
            port: None,
            account_unique_id: "".to_string(),
            account_field_map: AccountFieldMap {},
            org_unique_id: "".to_string(),
            org_field_map: OrgFieldMap {},
        }
    }
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
    pub port: Option<u16>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub account_unique_id: String,
    pub account_field_map: AccountFieldMap,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub org_unique_id: String,
    pub org_field_map: OrgFieldMap,
}

impl IamCertConfLdapResp {
    pub fn package_fitler_by_search_account(&self, user_or_display_name: &str) -> String {
        if let Some(search_base_filter) = self.account_field_map.search_base_filter.clone() {
            format!(
                "(&({})(|(cn=*{}*)({}=*{}*)))",
                search_base_filter, user_or_display_name, self.account_field_map.field_display_name, user_or_display_name
            )
        } else {
            todo!()
        }
    }
}
