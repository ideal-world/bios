use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_set_dto::RbumSetPathResp;
use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCpAccountInfoResp {
    pub account_id: String,
    pub account_name: String,
    pub tenant_name: Option<String>,
    pub roles: HashMap<String, String>,
    pub org: Vec<Vec<RbumSetPathResp>>,
    pub apps: Vec<IamCpAccountAppInfoResp>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCpAccountAppInfoResp {
    pub app_id: String,
    pub app_name: String,
    pub roles: HashMap<String, String>,
}
