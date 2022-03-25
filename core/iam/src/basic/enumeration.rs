use std::str::FromStr;

use derive_more::Display;
use sea_orm::strum::EnumString;
use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi::Enum;

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
            return IamCertTokenKind::TokenDefault;
        }
    }
}

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize, Enum, EnumString)]
pub enum IAMRelKind {
    IamAppTenant,
    IamAccountTenant,
    IamRoleTenant,
    IamRoleAccount,
    IamHttpResTenant,
    IamRoleHttpRes,
}
