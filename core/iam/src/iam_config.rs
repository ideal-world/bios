use std::fmt::Debug;
use std::sync::Mutex;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use bios_basic::rbum::rbum_config::RbumConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct IamConfig {
    pub rbum: RbumConfig,
    // token -> (token_kind, account_id)
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
    //  -> [res_uri##action, {st,et,accounts,roles,groups,apps,tenants}]
    pub cache_key_res_info: String,
    // time_stamp -> res_uri##action
    pub cache_key_res_changed_info_: String,
    pub cache_key_res_changed_expire_sec: usize,
    pub cache_key_async_task_status: String,
    pub mail_template_cert_activate_title: String,
    pub mail_template_cert_activate_content: String,
    pub mail_template_cert_login_title: String,
    pub mail_template_cert_login_content: String,
    pub phone_template_cert_activate_title: String,
    pub phone_template_cert_activate_content: String,
    pub phone_template_cert_login_title: String,
    pub phone_template_cert_login_content: String,

    pub ldap: IamLdapConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct IamLdapConfig {
    pub port: u16,
    pub dc: String,
    pub bind_dn: String,
    pub bind_password: String,

    pub client: Vec<LdapClientConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LdapClientConfig {
    pub code: TrimString,
    pub name: String,
    pub conn_uri: String,
    pub is_tls: bool,
    pub principal: TrimString,
    pub credentials: TrimString,
    pub base_dn: String,
    pub field_display_name: String,
    pub search_base_filter: String,
}

impl Default for IamLdapConfig {
    fn default() -> Self {
        IamLdapConfig {
            port: 10389,
            dc: "bios".to_string(),
            bind_dn: "CN=ldapadmin,DC=bios".to_string(),
            bind_password: "KDi234!ds".to_string(),
            client: vec![],
        }
    }
}

impl Default for IamConfig {
    fn default() -> Self {
        IamConfig {
            rbum: Default::default(),
            cache_key_token_info_: "iam:cache:token:info:".to_string(),
            cache_key_aksk_info_: "iam:cache:aksk:info:".to_string(),
            cache_key_account_rel_: "iam:cache:account:rel:".to_string(),
            cache_key_account_info_: "iam:cache:account:info:".to_string(),
            cache_key_role_info_: "iam:cache:role:info:".to_string(),
            cache_key_res_info: "iam:res:info".to_string(),
            cache_key_res_changed_info_: "iam:res:changed:info:".to_string(),
            cache_key_res_changed_expire_sec: 300,
            mail_template_cert_activate_title: "IAM Service Mail Credentials Activation".to_string(),
            mail_template_cert_activate_content: "Your account [{account_name}] is activating email credentials, verification code: {vcode}".to_string(),
            mail_template_cert_login_title: "IAM Service Mail login verification".to_string(),
            mail_template_cert_login_content: "Your account is trying to login, verification code: {vcode}".to_string(),
            phone_template_cert_activate_title: "IAM Service Phone Credentials Activation".to_string(),
            phone_template_cert_activate_content: "Your account [{account_name}] is activating phone credentials, verification code: {vcode}".to_string(),
            phone_template_cert_login_title: "Your account is trying to login, verification code: {vcode}".to_string(),
            phone_template_cert_login_content: "IAM Service Phone Credentials Activation".to_string(),
            ldap: IamLdapConfig::default(),
            cache_key_async_task_status: "iam:cache:task:status".to_string(),
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
    pub domain_iam_id: String,
    pub role_sys_admin_id: String,
    pub role_tenant_admin_id: String,
    pub role_app_admin_id: String,
}

lazy_static! {
    static ref BASIC_INFO: Mutex<Option<BasicInfo>> = Mutex::new(None);
}

pub struct IamBasicInfoManager;

impl IamBasicInfoManager {
    pub fn set(basic_info: BasicInfo) -> TardisResult<()> {
        let mut conf = BASIC_INFO.lock().map_err(|e| TardisError::internal_error(&format!("{:?}", e), ""))?;
        *conf = Some(basic_info);
        Ok(())
    }

    pub fn get_config<F, T>(fun: F) -> T
    where
        F: Fn(&BasicInfo) -> T,
    {
        let conf = BASIC_INFO.lock().unwrap_or_else(|e| panic!("iam basic config lock error: {:?}", e));
        let conf = conf.as_ref().unwrap_or_else(|| panic!("rbum config not set"));
        fun(conf)
    }
}

pub trait IamBasicConfigApi {
    fn iam_basic_kind_tenant_id(&self) -> String;
    fn iam_basic_kind_app_id(&self) -> String;
    fn iam_basic_kind_account_id(&self) -> String;
    fn iam_basic_kind_role_id(&self) -> String;
    fn iam_basic_kind_res_id(&self) -> String;
    fn iam_basic_domain_iam_id(&self) -> String;
    fn iam_basic_role_sys_admin_id(&self) -> String;
    fn iam_basic_role_tenant_admin_id(&self) -> String;
    fn iam_basic_role_app_admin_id(&self) -> String;
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

    fn iam_basic_domain_iam_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone())
    }

    fn iam_basic_role_sys_admin_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.role_sys_admin_id.clone())
    }

    fn iam_basic_role_tenant_admin_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.role_tenant_admin_id.clone())
    }

    fn iam_basic_role_app_admin_id(&self) -> String {
        IamBasicInfoManager::get_config(|conf| conf.role_app_admin_id.clone())
    }
}
