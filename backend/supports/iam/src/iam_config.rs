use std::fmt::Debug;
use std::sync::Mutex;

use bios_sdk_invoke::invoke_config::InvokeConfig;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tardis::basic::error::TardisError;

use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use bios_basic::rbum::rbum_config::RbumConfig;
use tardis::web::poem::http::HeaderName;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct IamConfig {
    pub rbum: RbumConfig,
    pub in_event: bool,
    pub invoke: InvokeConfig,
    pub app_res_data_guard_code: String,
    // token -> (token_kind, account_id)
    // accessToken(token_kind = TokenOauth2) -> (token_kind, rel_iam_item_id, ak, SetCateIds)
    pub cache_key_token_info_: String,
    // ak -> (sk,tenant_id,[appid])
    pub cache_key_aksk_info_: String,
    // account_id -> [token, (token_kind, add_time)]
    pub cache_key_account_rel_: String,
    // account_id -> {
    //     _: system or tenant context,
    //     <app_id>: app context,
    //     is_global<bool>:is global account
    // }
    pub cache_key_account_info_: String,
    // role_id -> iam_role
    pub cache_key_role_info_: String,
    pub cache_key_double_auth_info: String,
    pub cache_key_double_auth_expire_sec: usize,
    //  -> [res_uri##action, {st,et,accounts,roles,groups,apps,tenants}]
    pub cache_key_res_info: String,
    // time_stamp -> res_uri##action
    pub cache_key_res_changed_info_: String,
    pub cache_key_res_changed_expire_sec: usize,
    pub cache_key_async_task_status: String,
    pub cache_key_sync_ldap_status: String,
    pub cache_key_sync_ldap_task_lock: String,
    pub cache_key_gateway_rule_info_: String,
    pub mail_template_cert_activate_title: String,
    pub mail_template_cert_activate_content: String,
    pub mail_template_cert_login_title: String,
    pub mail_template_cert_login_content: String,
    pub mail_template_cert_random_pwd_title: String,
    pub mail_template_cert_random_pwd_content: String,
    //TODO remove?
    #[deprecated]
    pub phone_template_cert_activate_title: String,
    #[deprecated]
    pub phone_template_cert_activate_content: String,
    #[deprecated]
    pub phone_template_cert_login_title: String,
    #[deprecated]
    pub phone_template_cert_login_content: String,
    pub vcode_cd_in_sec: u32,
    pub sms_base_url: String,
    pub sms_path: String,
    pub sms_pwd_path: String,
    pub third_integration_config_key: String,
    pub third_integration_schedule_code: String,

    /// init custom role list
    pub init_role_list: Option<Vec<InitRole>>,
    pub init_menu_json_path: String,
    pub ldap: IamLdapConfig,

    pub spi: IamSpiConfig,
    pub iam_base_url: String,
    pub strict_security_mode: bool,
    pub crypto_conf: CryptoConf,

    pub gateway_openapi_path: String,
    pub crypto_pri_key: String,

    // OAuth2 相关配置
    pub oauth2_auth_code_expire_sec: u32,
    pub oauth2_access_token_default_expire_sec: u32,
    pub oauth2_refresh_token_expire_sec: u32,
    pub oauth2_require_pkce: bool,
    pub oauth2_allow_implicit_flow: bool,

    // open-api 插件配置
    pub openapi_plugin_time_range: String,
    pub openapi_plugin_limit: String,
    pub openapi_plugin_count: String,
    pub openapi_plugin_dynamic_route: String,
    pub openapi_plugin_allow_api_res: String,
    pub openapi_plugin_extand_header: String,
    pub openapi_plugin_state: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct IamLdapConfig {
    pub port: u16,
    pub dc: String,
    pub bind_dn: String,
    pub bind_password: String,
    /// Labor type translation map: code -> label
    pub labor_type_map: Option<std::collections::HashMap<String, String>>,
    /// Position translation map: code -> label
    pub position_map: Option<std::collections::HashMap<String, String>>,
}

impl Default for IamLdapConfig {
    fn default() -> Self {
        IamLdapConfig {
            port: 10389,
            dc: "bios".to_string(),
            bind_dn: "CN=ldapadmin,DC=bios".to_string(),
            bind_password: "KDi234!ds".to_string(),
            labor_type_map: None,
            position_map: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct IamSpiConfig {
    pub schedule_url: String,
    pub search_url: String,
    pub log_url: String,
    pub search_account_tag: String,
    pub stats_orgs_prefix: String,
    pub kv_url: String,
    pub kv_tenant_prefix: String,
    pub kv_account_prefix: String,
    pub kv_app_prefix: String,
    pub kv_apps_prefix: String,
    pub kv_orgs_prefix: String,
    pub kv_role_prefix: String,
    pub kv_api_call_count_prefix: String,

    pub owner: String,
}
impl Default for IamSpiConfig {
    fn default() -> Self {
        IamSpiConfig {
            schedule_url: "http://127.0.0.1:8080/schedule".to_string(),
            search_url: "http://127.0.0.1:8080/spi-search".to_string(),
            log_url: "http://127.0.0.1:8080/spi-log".to_string(),
            search_account_tag: "iam_account".to_string(),
            stats_orgs_prefix: "iam_orgs".to_string(),
            kv_url: "http://127.0.0.1:8080/spi-kv".to_string(),
            kv_tenant_prefix: "iam_tenant".to_string(),
            kv_account_prefix: "iam_account".to_string(),
            kv_app_prefix: "iam_app".to_string(),
            kv_apps_prefix: "iam_apps".to_string(),
            kv_orgs_prefix: "iam_orgs".to_string(),
            kv_role_prefix: "iam_role".to_string(),
            kv_api_call_count_prefix: "api_call_count".to_string(),
            owner: "".to_string(),
        }
    }
}

impl Default for IamConfig {
    fn default() -> Self {
        IamConfig {
            rbum: Default::default(),
            in_event: false,
            invoke: InvokeConfig::default(),
            app_res_data_guard_code: "5/*/app*data_guard*all".to_string(),
            cache_key_token_info_: "iam:cache:token:info:".to_string(),
            cache_key_aksk_info_: "iam:cache:aksk:info:".to_string(),
            cache_key_account_rel_: "iam:cache:account:rel:".to_string(),
            cache_key_account_info_: "iam:cache:account:info:".to_string(),
            cache_key_role_info_: "iam:cache:role:info:".to_string(),
            // ..:<account_id>
            cache_key_double_auth_info: "iam:cache:double_auth:info:".to_string(),
            cache_key_double_auth_expire_sec: 300,
            cache_key_res_info: "iam:res:info".to_string(),
            cache_key_res_changed_info_: "iam:res:changed:info:".to_string(),
            cache_key_res_changed_expire_sec: 300,
            mail_template_cert_activate_title: "IAM Service Mail Credentials Activation".to_string(),
            mail_template_cert_activate_content: "Your account [{account_name}] is activating email credentials, verification code: {vcode}".to_string(),
            mail_template_cert_login_title: "IAM Service Mail login verification".to_string(),
            mail_template_cert_login_content: "Your account is trying to login, verification code: {vcode}".to_string(),
            mail_template_cert_random_pwd_title: "IAM Service Mail password verification".to_string(),
            mail_template_cert_random_pwd_content: "Your account has just been created, verification password: {pwd}".to_string(),
            phone_template_cert_activate_title: "IAM Service Phone Credentials Activation".to_string(),
            phone_template_cert_activate_content: "Your account [{account_name}] is activating phone credentials, verification code: {vcode}".to_string(),
            phone_template_cert_login_title: "Your account is trying to login, verification code: {vcode}".to_string(),
            phone_template_cert_login_content: "IAM Service Phone Credentials Activation".to_string(),
            init_menu_json_path: "config/init-menu-default.json".to_string(),
            init_role_list: None,
            ldap: IamLdapConfig::default(),
            cache_key_async_task_status: "iam:cache:task:status".to_string(),
            cache_key_sync_ldap_status: "iam:cache:sync:ldap:status".to_string(),
            cache_key_sync_ldap_task_lock: "iam:cache:sync:ldap:taskId".to_string(),
            sms_base_url: "http://reach:8080".to_string(),
            sms_path: "cc/msg/vcode".to_string(),
            sms_pwd_path: "cc/msg/pwd".to_string(),
            third_integration_config_key: "iam:third:integration:config:key".to_string(),
            third_integration_schedule_code: "iam:third:integration".to_string(),
            iam_base_url: "http://127.0.0.1:8080/iam".to_string(),
            spi: Default::default(),
            strict_security_mode: false,
            crypto_pri_key: "".to_string(),
            crypto_conf: CryptoConf::default(),
            cache_key_gateway_rule_info_: "sg:plugin:".to_string(),
            gateway_openapi_path: "/op-api".to_string(),
            vcode_cd_in_sec: crate::iam_constants::DEFAULT_V_CODE_CD_IN_SEC,

            // OAuth2 默认配置
            oauth2_auth_code_expire_sec: 600,                      // 10分钟
            oauth2_access_token_default_expire_sec: 7 * 24 * 3600, // 7天
            oauth2_refresh_token_expire_sec: 30 * 24 * 3600,       // 30天
            oauth2_require_pkce: false,
            oauth2_allow_implicit_flow: false,

            // open-api 插件配置
            openapi_plugin_time_range: "redis-time-range:opres-time-range".to_string(),
            openapi_plugin_limit: "redis-limit:opres-limit".to_string(),
            openapi_plugin_count: "redis-count:opres-count".to_string(),
            openapi_plugin_dynamic_route: "redis-dynamic-route:opres-dynamic-route".to_string(),
            openapi_plugin_allow_api_res: "redis-allow-api:opres-allow-api".to_string(),
            openapi_plugin_extand_header: "redis-header-expand:opres-header-expand".to_string(),
            openapi_plugin_state: "redis-status:opres-status".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CryptoConf {
    pub head_key_crypto: String,
    pub auth_url: String,
}
impl CryptoConf {
    pub fn get_crypto_header_name(&self) -> TardisResult<HeaderName> {
        HeaderName::try_from(&self.head_key_crypto)
            .map_err(|e| TardisError::custom("500", &format!("[Iam] head_key_crypto config error,can't be HeaderName: {e}"), "500-config-parse-error"))
    }
}

impl Default for CryptoConf {
    fn default() -> Self {
        CryptoConf {
            head_key_crypto: "Bios-Crypto".to_string(),
            auth_url: "http://127.0.0.1:8080/auth".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BasicInfo {
    pub kind_tenant_id: String,
    pub kind_app_id: String,
    pub kind_account_id: String,
    pub kind_role_id: String,
    pub kind_res_id: String,
    pub kind_sub_deploy_id: String,
    pub domain_iam_id: String,
    pub role_sys_admin_id: String,
    pub role_tenant_admin_id: String,
    pub role_tenant_audit_id: String,
    pub role_app_admin_id: String,
    pub role_app_read_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InitRole {
    pub code: String,
    pub kind: crate::iam_enumeration::IamRoleKind,
}

lazy_static! {
    static ref BASIC_INFO: Mutex<Option<BasicInfo>> = Mutex::new(None);
}

pub struct IamBasicInfoManager;

impl IamBasicInfoManager {
    pub fn set(basic_info: BasicInfo) -> TardisResult<()> {
        let mut conf = BASIC_INFO.lock().map_err(|e| TardisError::internal_error(&format!("{e:?}"), ""))?;
        *conf = Some(basic_info);
        Ok(())
    }

    pub fn get_config<F, T>(fun: F) -> T
    where
        F: Fn(&BasicInfo) -> T,
    {
        let conf = BASIC_INFO.lock().unwrap_or_else(|e| panic!("iam basic info lock error: {e:?}"));
        let conf = conf.as_ref().unwrap_or_else(|| panic!("iam basic info not set"));
        fun(conf)
    }
}

pub trait IamBasicConfigApi {
    fn iam_basic_kind_tenant_id(&self) -> String;
    fn iam_basic_kind_app_id(&self) -> String;
    fn iam_basic_kind_account_id(&self) -> String;
    fn iam_basic_kind_role_id(&self) -> String;
    fn iam_basic_kind_res_id(&self) -> String;
    fn iam_basic_kind_sub_deploy_id(&self) -> String;
    fn iam_basic_domain_iam_id(&self) -> String;
    fn iam_basic_role_sys_admin_id(&self) -> String;
    fn iam_basic_role_tenant_admin_id(&self) -> String;
    fn iam_basic_role_tenant_audit_id(&self) -> String;
    fn iam_basic_role_app_admin_id(&self) -> String;
    fn iam_basic_role_app_read_id(&self) -> String;
}

impl IamBasicConfigApi for TardisFunsInst {
    fn iam_basic_kind_tenant_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.kind_tenant_id.clone())
    }

    fn iam_basic_kind_app_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.kind_app_id.clone())
    }

    fn iam_basic_kind_account_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.kind_account_id.clone())
    }

    fn iam_basic_kind_role_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.kind_role_id.clone())
    }

    fn iam_basic_kind_res_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.kind_res_id.clone())
    }

    fn iam_basic_kind_sub_deploy_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.kind_sub_deploy_id.clone())
    }

    fn iam_basic_domain_iam_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone())
    }

    fn iam_basic_role_sys_admin_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.role_sys_admin_id.clone())
    }

    fn iam_basic_role_tenant_admin_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.role_tenant_admin_id.clone())
    }

    fn iam_basic_role_tenant_audit_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.role_tenant_audit_id.clone())
    }

    fn iam_basic_role_app_admin_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.role_app_admin_id.clone())
    }

    fn iam_basic_role_app_read_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.role_app_read_id.clone())
    }
}
