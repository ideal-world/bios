use std::str::FromStr;

use serde::{Deserialize, Serialize};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm;
use tardis::db::sea_orm::{DbErr, QueryResult, TryGetError, TryGetable};
use tardis::derive_more::Display;
use tardis::web::poem_openapi;

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum IamRoleKind {
    System,
    Tenant,
    App,
}

impl IamRoleKind {
    pub fn from_int(s: i16) -> TardisResult<IamRoleKind> {
        match s {
            0 => Ok(IamRoleKind::System),
            1 => Ok(IamRoleKind::Tenant),
            2 => Ok(IamRoleKind::App),
            _ => Err(TardisError::format_error(&format!("invalid IamRoleKind: {s}"), "406-rbum-*-enum-init-error")),
        }
    }

    pub fn to_int(&self) -> i16 {
        match self {
            IamRoleKind::System => 0,
            IamRoleKind::Tenant => 1,
            IamRoleKind::App => 2,
        }
    }
}

impl TryGetable for IamRoleKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = i16::try_get(res, pre, col)?;
        IamRoleKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implement")
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum IamCertKernelKind {
    UserPwd,
    MailVCode,
    PhoneVCode,
    AkSk,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum IamCertExtKind {
    Ldap,
    OAuth2,
    /// No configuration exists,can't login in ,\
    /// supplier can be "gitlab/cmbd-pwd/cmbd-ssh"
    ThirdParty,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum IamCertOAuth2Supplier {
    // Weibo,
    Github,
    WechatMp,
}

impl IamCertOAuth2Supplier {
    pub fn parse(kind: &str) -> TardisResult<IamCertOAuth2Supplier> {
        IamCertOAuth2Supplier::from_str(kind).map_err(|_| TardisError::format_error(&format!("not support OAuth2 kind: {kind}",), "404-iam-cert-oauth-kind-not-exist"))
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum IamCertTokenKind {
    TokenDefault,
    TokenPc,
    TokenPhone,
    TokenPad,
    TokenOauth2,
}

impl IamCertTokenKind {
    pub fn parse(kind: &Option<String>) -> IamCertTokenKind {
        if let Some(kind) = kind {
            IamCertTokenKind::from_str(kind).unwrap_or(IamCertTokenKind::TokenDefault)
        } else {
            IamCertTokenKind::TokenDefault
        }
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum IamRelKind {
    IamAccountRole,
    IamResRole,
    IamAccountApp,
    IamResApi,
    IamAccountRel,
    IamCertRel,
    IamOrgRel,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum IamResKind {
    Menu,
    Api,
    Ele,
}

impl IamResKind {
    pub fn from_int(s: i16) -> TardisResult<IamResKind> {
        match s {
            0 => Ok(IamResKind::Menu),
            1 => Ok(IamResKind::Api),
            2 => Ok(IamResKind::Ele),
            _ => Err(TardisError::format_error(&format!("invalid IamResKind: {s}"), "406-rbum-*-enum-init-error")),
        }
    }

    pub fn to_int(&self) -> i16 {
        match self {
            IamResKind::Menu => 0,
            IamResKind::Api => 1,
            IamResKind::Ele => 2,
        }
    }
}

impl TryGetable for IamResKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = i16::try_get(res, pre, col)?;
        IamResKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implement")
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum IamSetKind {
    Org,
    Res,
    Apps,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum IamSetCateKind {
    Root,
    System,
    Tenant,
    App,
}

impl IamSetCateKind {
    pub fn parse(kind: &str) -> TardisResult<IamSetCateKind> {
        IamSetCateKind::from_str(kind).map_err(|_| TardisError::format_error(&format!("not support SetCate kind: {kind}"), "404-iam-cert-set-cate-kind-not-exist"))
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum Oauth2GrantType {
    AuthorizationCode,
    Password,
    ClientCredentials,
}

impl Oauth2GrantType {
    pub fn parse(kind: &str) -> TardisResult<Oauth2GrantType> {
        Oauth2GrantType::from_str(kind).map_err(|_| TardisError::format_error(&format!("not support OAuth2 kind: {kind}"), "404-iam-cert-oauth-kind-not-exist"))
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum WayToAdd {
    ///同步账号同步凭证
    SynchronizeCert,
    ///同步账号不凭证
    NoSynchronizeCert,
}

impl Default for WayToAdd {
    fn default() -> Self {
        WayToAdd::SynchronizeCert
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum WayToDelete {
    ///什么也不做
    DoNotDelete,
    ///取消授权
    DeleteCert,
    ///禁用账号
    Disable,
    ///同步删除账号凭证
    DeleteAccount,
}
impl Default for WayToDelete {
    fn default() -> Self {
        WayToDelete::DoNotDelete
    }
}
