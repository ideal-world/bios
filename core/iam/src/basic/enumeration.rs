use derive_more::Display;
use sea_orm::strum::EnumString;
use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi::Enum;

#[derive(Display, Clone, Debug, PartialEq, Deserialize, Serialize, Enum, EnumString)]
pub enum IamIdentKind {
    UserPwd,
    MailVCode,
    PhoneVCode,
}
