use std::str::FromStr;

use derive_more::Display;
use sea_orm::strum::EnumString;
use serde::{Deserialize, Serialize};
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
    IamTenantApp,
    IamTenantAccount,
    IamTenantRole,
    IamAccountRole,
    IamTenantHttpRes,
    IamHttpResRole,
}
