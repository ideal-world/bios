use std::fmt::Debug;
use std::sync::Mutex;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;

use bios_basic::rbum::rbum_config::RbumConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct IamConfig {
    pub rbum: RbumConfig,
    // token -> (token_kind, account_id)
    pub cache_key_token_info_: String,
    // account_id -> [token, (token_kind, add_time)]
    pub cache_key_account_rel_: String,
    // account_id -> {
    //     _: system or tenant context,
    //     <app_id>: app context,
    // }
    pub cache_key_account_info_: String,
    // role_id -> iam_role
    pub cache_key_role_info_: String,
    pub mail_template_cert_activate_title: String,
    pub mail_template_cert_activate_content: String,
    pub mail_template_cert_login_title: String,
    pub mail_template_cert_login_content: String,
}

impl Default for IamConfig {
    fn default() -> Self {
        IamConfig {
            rbum: Default::default(),
            cache_key_token_info_: "iam:cache:token:info:".to_string(),
            cache_key_account_rel_: "iam:cache:account:rel:".to_string(),
            cache_key_account_info_: "iam:cache:account:info:".to_string(),
            cache_key_role_info_: "iam:cache:role:info:".to_string(),
            mail_template_cert_activate_title: "IAM Service Mail Credentials Activation".to_string(),
            mail_template_cert_activate_content: "Your account [{account_name}] is activating email credentials, verification code: {vcode}".to_string(),
            mail_template_cert_login_title: "IAM Service Mail login verification".to_string(),
            mail_template_cert_login_content: "Your account is trying to login, verification code: {vcode}".to_string(),
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
        let mut conf = BASIC_INFO.lock().map_err(|e| TardisError::InternalError(format!("{:?}", e)))?;
        *conf = Some(basic_info);
        Ok(())
    }

    pub fn get() -> BasicInfo {
        let conf = BASIC_INFO.lock().expect("Basic info not set");
        conf.as_ref().expect("Basic info not set").clone()
    }
}
