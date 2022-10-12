use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpUserPwdLoginReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sk: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub flag: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpMailVCodeLoginGenVCodeReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpMailVCodeLoginReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub vcode: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub flag: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpPhoneVCodeLoginGenVCodeReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub phone: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpPhoneVCodeLoginSendVCodeReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub phone: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub vcode: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub flag: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpOAuth2LoginReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpLdapLoginReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub password: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpUserPwdBindWithLdapReq {
    pub bind_user_pwd: IamCpUserPwdBindReq,
    pub ldap_login: IamCpLdapLoginReq,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpUserPwdCheckReq{
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code:TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id:String,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpUserPwdBindReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sk: TrimString,
}
