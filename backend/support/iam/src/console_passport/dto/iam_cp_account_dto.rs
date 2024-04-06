use std::collections::HashMap;

use crate::basic::dto::iam_account_dto::IamAccountAttrResp;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpAccountInfoResp {
    pub account_id: String,
    pub account_name: String,
    pub account_icon: String,
    pub tenant_id: Option<String>,
    pub tenant_name: Option<String>,
    pub roles: HashMap<String, String>,
    pub org: Vec<String>,
    pub apps: Vec<IamCpAccountAppInfoResp>,
    pub certs: HashMap<String, String>,
    pub exts: Vec<IamAccountAttrResp>,
    pub scope_level: RbumScopeLevelKind,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpAccountAppInfoResp {
    pub app_id: String,
    pub app_name: String,
    pub app_icon: String,
    pub roles: HashMap<String, String>,
}
