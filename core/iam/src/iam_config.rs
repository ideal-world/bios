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
}

impl Default for IamConfig {
    fn default() -> Self {
        IamConfig { rbum: Default::default() }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BasicInfo {
    pub kind_tenant_id: String,
    pub kind_app_id: String,
    pub kind_account_id: String,
    pub kind_role_id: String,
    pub kind_http_res_id: String,
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
