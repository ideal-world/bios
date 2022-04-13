use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi::Object;

#[derive(Object, Serialize, Deserialize, Debug)]
#[oai(rename_all = "camelCase")]
pub struct IamUserPwdCertAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sk: TrimString,
}

#[derive(Object, Serialize, Deserialize, Debug)]
#[oai(rename_all = "camelCase")]
pub struct IamUserPwdCertModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub original_sk: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub new_sk: TrimString,
}

#[derive(Object, Serialize, Deserialize, Debug)]
#[oai(rename_all = "camelCase")]
pub struct IamMailVCodeCertAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
}

#[derive(Object, Serialize, Deserialize, Debug)]
#[oai(rename_all = "camelCase")]
pub struct IamMailVCodeCertModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
}

#[derive(Object, Serialize, Deserialize, Debug)]
#[oai(rename_all = "camelCase")]
pub struct IamPhoneVCodeCertAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
}

#[derive(Object, Serialize, Deserialize, Debug)]
#[oai(rename_all = "camelCase")]
pub struct IamPhoneVCodeCertModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
}

