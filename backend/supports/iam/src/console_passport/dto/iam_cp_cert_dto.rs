use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::web::poem_openapi;

use crate::iam_enumeration::{OAuth2ResponseType, Oauth2GrantType};

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
pub struct IamCpExistMailVCodeReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpMailVCodeLoginGenVCodeReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpMailVCodeLoginReq {
    #[oai(validator(min_length = "2", max_length = "255", custom = "tardis::web::web_validation::Mail"))]
    pub mail: String,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub vcode: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: Option<String>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub flag: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpPhoneVCodeLoginGenVCodeReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub phone: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpExistPhoneVCodeReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub phone: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpPhoneVCodeLoginSendVCodeReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub phone: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub vcode: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: Option<String>,
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
    pub tenant_id: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpUserPwdBindWithLdapReq {
    pub bind_user_pwd: IamCpUserPwdBindReq,
    pub ldap_login: IamCpLdapLoginReq,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpUserPwdCheckReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: TrimString,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: TrimString,
    #[oai(validator(min_length = "0", max_length = "255"))]
    pub tenant_id: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpUserPwdBindReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub ak: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sk: TrimString,
}

/// 通过现有 token 刷新 Redis 账号上下文至目标租户或平台
///
/// token 保持不变，仅将 `cache_key_account_info_` 中的上下文数据刷新为目标租户维度。
/// `tenant_id` 为 `None` 时，切换到平台级全局上下文（own_paths = ""）。
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpTokenSwitchReq {
    /// 当前有效的 token，刷新后继续使用，不会失效
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub token: String,
    /// 目标租户 ID；为 None 时切换到平台级全局上下文
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub tenant_id: Option<String>,
}

// OAuth2 Service DTOs for Console Passport
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpOAuth2ServiceAuthorizeReq {
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub client_id: TrimString,
    pub state: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub scope: TrimString,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub redirect_uri: TrimString,
    #[oai(default)]
    pub response_type: OAuth2ResponseType,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpOAuth2ServiceTokenReq {
    pub grant_type: Oauth2GrantType,
    pub code: Option<String>,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCpOAuth2ServiceAuthorizeResp {
    pub redirect_url: String,
}
