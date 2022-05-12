use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi::Object;

use crate::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};

#[derive(Object, Serialize, Deserialize, Debug)]
pub struct IamCsTenantAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_name: TrimString,
    #[oai(validator(min_length = "2", max_length = "1000"))]
    pub tenant_icon: Option<String>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_contact_phone: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_note: Option<String>,

    #[oai(validator(min_length = "2", max_length = "255"))]
    pub admin_username: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub admin_password: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub admin_name: TrimString,
    
    pub cert_conf_by_user_pwd: IamUserPwdCertConfAddOrModifyReq,
    pub cert_conf_by_phone_vcode: Option<IamPhoneVCodeCertConfAddOrModifyReq>,
    pub cert_conf_by_mail_vcode: Option<IamMailVCodeCertConfAddOrModifyReq>,

    pub disabled: Option<bool>,
}
