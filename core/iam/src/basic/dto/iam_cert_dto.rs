use bios_basic::rbum::rbum_enumeration::RbumCertStatusKind;
use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamContextFetchReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub token: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub app_id: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertPwdNewReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub original_sk: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub new_sk: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertUserPwdAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sk: TrimString,
    pub status: Option<RbumCertStatusKind>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertUserPwdModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub original_sk: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub new_sk: TrimString,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertUserPwdRestReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub new_sk: TrimString,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertUserPwdValidateSkReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sk: TrimString,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertMailVCodeAddReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertMailVCodeResendActivationReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertMailVCodeActivateReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub vcode: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertPhoneVCodeAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub phone: TrimString,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertPhoneVCodeBindReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub phone: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub vcode: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertExtAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sk: Option<String>,
    pub ext: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertManageAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sk: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub supplier: String,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ext: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertManageModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sk: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub ext: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertOAuth2AddOrModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub open_id: TrimString,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertLdapAddOrModifyReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub dn: TrimString,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertAkSkAddReq {
    pub ak: String,
    pub sk: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCertAkSkResp{
    pub id:String,
    pub ak: String,
    pub sk: String,
}
