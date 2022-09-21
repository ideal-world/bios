use std::str::FromStr;

use serde::{Deserialize, Serialize};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm;
use tardis::db::sea_orm::{DbErr, QueryResult, TryGetError, TryGetable};
use tardis::derive_more::Display;
use tardis::web::poem_openapi;

#[derive(poem_openapi::Tags, Display, Debug)]
pub enum Tag {
    #[oai(rename = "Common Console")]
    Common,
    #[oai(rename = "Tenant Console")]
    Tenant,
    #[oai(rename = "App Console")]
    App,
    #[oai(rename = "System Console")]
    System,
    #[oai(rename = "Passport Console")]
    Passport,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum IamRoleKind {
    System,
    Tenant,
    App,
}

impl IamRoleKind {
    pub fn from_int(s: u8) -> TardisResult<IamRoleKind> {
        match s {
            0 => Ok(IamRoleKind::System),
            1 => Ok(IamRoleKind::Tenant),
            2 => Ok(IamRoleKind::App),
            _ => Err(TardisError::format_error(&format!("invalid IamRoleKind: {}", s), "406-rbum-*-enum-init-error")),
        }
    }

    pub fn to_int(&self) -> u8 {
        match self {
            IamRoleKind::System => 0,
            IamRoleKind::Tenant => 1,
            IamRoleKind::App => 2,
        }
    }
}

impl TryGetable for IamRoleKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = u8::try_get(res, pre, col)?;
        IamRoleKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum IamCertKernelKind {
    UserPwd,
    MailVCode,
    PhoneVCode,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum IamCertManageKind {
    ManageUserPwd,
    ManageUserVisa,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum IamCertExtKind {
    Gitlab,
    Github,
    Wechat,
    // TODO
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, sea_orm::strum::EnumString)]
pub enum IamCertTokenKind {
    TokenDefault,
    TokenPc,
    TokenPhone,
    TokenPad,
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
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum IamResKind {
    Menu,
    Api,
    Ele,
}

impl IamResKind {
    pub fn from_int(s: u8) -> TardisResult<IamResKind> {
        match s {
            0 => Ok(IamResKind::Menu),
            1 => Ok(IamResKind::Api),
            2 => Ok(IamResKind::Ele),
            _ => Err(TardisError::format_error(&format!("invalid IamResKind: {}", s), "406-rbum-*-enum-init-error")),
        }
    }

    pub fn to_int(&self) -> u8 {
        match self {
            IamResKind::Menu => 0,
            IamResKind::Api => 1,
            IamResKind::Ele => 2,
        }
    }
}

impl TryGetable for IamResKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = u8::try_get(res, pre, col)?;
        IamResKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum IamSetKind {
    Org,
    Res,
    Apps,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum IamSetCateKind {
    Root,
    System,
    Tenant,
    App,
}
