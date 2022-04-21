use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct LoginResp {
    pub id: String,
    pub name: String,
    pub token: String,
    pub roles: HashMap<String, String>,
    pub groups: HashMap<String, String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCpUserPwdLoginReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sk: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    // TODO tmp
    pub app_id: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub flag: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCpMailVCodeLoginReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub flag: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCpPhoneVCodeLoginReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub flag: Option<String>,
}
