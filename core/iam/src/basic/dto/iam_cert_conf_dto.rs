use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamUserPwdCertConfAddOrModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_rule: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub sk_note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub sk_rule: Option<String>,
    pub ext: Option<String>,
    pub repeatable: Option<bool>,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub expire_sec: Option<u32>,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub sk_lock_cycle_sec: Option<u32>,
    pub sk_lock_err_times: Option<u8>,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub sk_lock_duration_sec: Option<u32>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamUserPwdCertConfInfo {
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
    pub sk_lock_cycle_sec: u32,
    pub sk_lock_err_times: u8,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub sk_lock_duration_sec: u32,
    pub repeatable: bool,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub expire_sec: u32,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamMailVCodeCertConfAddOrModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_rule: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone)]
pub struct IamPhoneVCodeCertConfAddOrModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_note: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ak_rule: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamTokenCertConfAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    pub coexist_num: u32,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub expire_sec: Option<u32>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamTokenCertConfModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    pub coexist_num: Option<u32>,
    #[oai(validator(minimum(value = "1", exclusive = "false")))]
    pub expire_sec: Option<u32>,
}
