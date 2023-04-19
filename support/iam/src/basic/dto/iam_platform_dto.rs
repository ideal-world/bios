use crate::basic::dto::iam_cert_conf_dto::IamCertConfUserPwdResp;
use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

use super::iam_cert_conf_dto::{IamCertConfUserPwdAddOrModifyReq};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamPlatformAggModifyReq {
    pub cert_conf_by_user_pwd: Option<IamCertConfUserPwdAddOrModifyReq>,
    pub cert_conf_by_phone_vcode: Option<bool>,
    pub cert_conf_by_mail_vcode: Option<bool>,
    
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamPlatformAggDetailResp {
    pub cert_conf_by_user_pwd: IamCertConfUserPwdResp,
    pub cert_conf_by_phone_vcode: bool,
    pub cert_conf_by_mail_vcode: bool,
}
