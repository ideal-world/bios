use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamContextFetchReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub token: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub app_id: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamPwdNewReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub original_sk: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub new_sk: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: Option<String>,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamUserPwdCertAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sk: TrimString,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamUserPwdCertModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub original_sk: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub new_sk: TrimString,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamUserPwdCertRestReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub new_sk: TrimString,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamMailVCodeCertAddReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamMailVCodeCertResendActivationReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamMailVCodeCertActivateReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub vcode: String,
}

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamPhoneVCodeCertAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub phone: TrimString,
}
