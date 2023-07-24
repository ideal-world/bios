use crate::basic::dto::iam_cert_conf_dto::IamCertConfUserPwdResp;
use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

use super::{iam_cert_conf_dto::IamCertConfUserPwdAddOrModifyReq, iam_config_dto::IamConfigAggOrModifyReq, iam_config_dto::IamConfigSummaryResp};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamPlatformConfigReq {
    pub cert_conf_by_mail_vcode: Option<bool>,
    pub cert_conf_by_phone_vcode: Option<bool>,
    pub token_default_coexist_num: Option<i16>,
    pub cert_conf_by_user_pwd: Option<IamCertConfUserPwdAddOrModifyReq>,
    pub config: Option<Vec<IamConfigAggOrModifyReq>>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamPlatformConfigResp {
    pub strict_security_mode: bool,
    pub cert_conf_by_user_pwd: IamCertConfUserPwdResp,
    pub cert_conf_by_phone_vcode: bool,
    pub cert_conf_by_mail_vcode: bool,
    pub token_default_coexist_num: i16,
    pub config: Vec<IamConfigSummaryResp>,
}
