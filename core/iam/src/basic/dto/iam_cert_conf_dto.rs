use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamUserPwdCertConfAddOrModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_rule: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub sk_note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub sk_rule: Option<String>,
    pub repeatable: Option<bool>,
    pub expire_sec: Option<i32>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamMailVCodeCertConfAddOrModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_rule: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamPhoneVCodeCertConfAddOrModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_rule: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamTokenCertConfAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    pub coexist_num: i32,
    pub expire_sec: Option<i32>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamTokenCertConfModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    pub coexist_num: Option<i32>,
    pub expire_sec: Option<i32>,
}
