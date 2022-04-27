use std::str::FromStr;

use derive_more::Display;
use sea_orm::strum::EnumString;
use sea_orm::{DbErr, QueryResult, TryGetError, TryGetable};
use serde::{Deserialize, Serialize};
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::web::poem_openapi::Enum;
use tardis::web::poem_openapi::Tags;

#[derive(Tags, Display, Debug)]
pub enum Tag {
    #[oai(rename = "Tenant Console")]
    Tenant,
    #[oai(rename = "App Console")]
    App,
    #[oai(rename = "System Console")]
    System,
    #[oai(rename = "Passport")]
    Passport,
}

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize, Enum, EnumString)]
pub enum IamCertKind {
    UserPwd,
    MailVCode,
    PhoneVCode,
}

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize, Enum, EnumString)]
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

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize, Enum, EnumString)]
pub enum IAMRelKind {
    IamAccountRole,
    IamResRole,
}

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize, Enum)]
pub enum IamResKind {
    MENU,
    API,
    ELEMENT,
}

impl IamResKind {
    pub fn from_int(s: u8) -> TardisResult<IamResKind> {
        match s {
            0 => Ok(IamResKind::MENU),
            1 => Ok(IamResKind::API),
            2 => Ok(IamResKind::ELEMENT),
            _ => Err(TardisError::FormatError(format!("Invalid IamResKind: {}", s))),
        }
    }

    pub fn to_int(&self) -> u8 {
        match self {
            IamResKind::MENU => 0,
            IamResKind::API => 1,
            IamResKind::ELEMENT => 2,
        }
    }
}

impl TryGetable for IamResKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = u8::try_get(res, pre, col)?;
        IamResKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{}:{}", pre, col))))
    }
}
