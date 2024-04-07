use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use tardis::{
    basic::error::TardisError,
    db::sea_orm::{self, DeriveActiveEnum, EnumIter},
    web::poem_openapi,
};
#[repr(u8)]
#[derive(Debug, poem_openapi::Enum, EnumIter, Clone, Copy, DeriveActiveEnum, PartialEq, Eq, Serialize, Deserialize, Hash, Default)]
#[oai(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "String(Some(255))")]
pub enum ReachChannelKind {
    #[sea_orm(string_value = "SMS")]
    #[default]
    Sms,
    #[sea_orm(string_value = "EMAIL")]
    Email,
    #[sea_orm(string_value = "INBOX")]
    Inbox,
    #[sea_orm(string_value = "WECHAT")]
    Wechat,
    #[sea_orm(string_value = "DINGTALK")]
    DingTalk,
    #[sea_orm(string_value = "PUSH")]
    Push,
    #[sea_orm(string_value = "WEB_HOOK")]
    WebHook,
}

impl Display for ReachChannelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReachChannelKind::Sms => write!(f, "SMS"),
            ReachChannelKind::Email => write!(f, "EMAIL"),
            ReachChannelKind::Inbox => write!(f, "INBOX"),
            ReachChannelKind::Wechat => write!(f, "WECHAT"),
            ReachChannelKind::DingTalk => write!(f, "DINGTALK"),
            ReachChannelKind::Push => write!(f, "PUSH"),
            ReachChannelKind::WebHook => write!(f, "WEB_HOOK"),
        }
    }
}
impl FromStr for ReachChannelKind {
    type Err = TardisError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "SMS" => Ok(Self::Sms),
            "EMAIL" => Ok(Self::Email),
            "INBOX" => Ok(Self::Inbox),
            "WECHAT" => Ok(Self::Wechat),
            "DINGTALK" => Ok(Self::DingTalk),
            "PUSH" => Ok(Self::Push),
            "WEB_HOOK" => Ok(Self::WebHook),
            _ => Err(TardisError::bad_request(&format!("invalid ReachChannelKind: {}", s), "400-reach-invalid-param")),
        }
    }
}

#[derive(Debug, poem_openapi::Enum, EnumIter, Clone, Copy, DeriveActiveEnum, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[oai(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "String(Some(255))")]
pub enum ReachReceiveKind {
    #[sea_orm(string_value = "ACCOUNT")]
    Account,
    #[sea_orm(string_value = "ROLE")]
    Role,
    #[sea_orm(string_value = "APP")]
    App,
    #[sea_orm(string_value = "TENANT")]
    Tenant,
}

#[derive(Debug, poem_openapi::Enum, EnumIter, Clone, Copy, DeriveActiveEnum, PartialEq, Eq, Serialize, Deserialize, Default)]
#[oai(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "String(Some(255))")]
pub enum ReachStatusKind {
    #[sea_orm(string_value = "DRAFT")]
    Draft,
    #[sea_orm(string_value = "PENDING")]
    #[default]
    /// 定时消息未发送时的状态
    Pending,
    /// 非 [ReachChannelKind::Inbox] 类型的发送中的状态
    #[sea_orm(string_value = "SENDING")]
    Sending,
    #[sea_orm(string_value = "SEND_SUCCESS")]
    SendSuccess,
    #[sea_orm(string_value = "ALL_DELEVERY")]
    /// 多用于站内信
    AllDelevery,
    #[sea_orm(string_value = "FAIL")]
    /// 非 [ReachChannelKind::Inbox] 类型的失败的状态
    Fail,
}

#[derive(Debug, poem_openapi::Enum, EnumIter, Clone, Copy, DeriveActiveEnum, PartialEq, Eq, Serialize, Deserialize)]
#[oai(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "String(Some(255))")]
pub enum ReachDndStrategyKind {
    #[sea_orm(string_value = "IGNORE")]
    Ignore,
    #[sea_orm(string_value = "RETRY_ONCE")]
    Delay,
}

#[derive(Debug, poem_openapi::Enum, EnumIter, Clone, Copy, DeriveActiveEnum, PartialEq, Eq, Serialize, Deserialize, Default)]
#[oai(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "String(Some(255))")]
pub enum ReachLevelKind {
    #[sea_orm(string_value = "URGENT")]
    Urgent,
    #[sea_orm(string_value = "HIGH")]
    High,
    #[sea_orm(string_value = "NORMAL")]
    #[default]
    Normal,
    #[sea_orm(string_value = "LOW")]
    Low,
}

#[derive(Debug, poem_openapi::Enum, EnumIter, Clone, Copy, DeriveActiveEnum, PartialEq, Eq, Serialize, Deserialize, Default)]
#[oai(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "String(Some(255))")]
pub enum ReachTimeoutStrategyKind {
    #[sea_orm(string_value = "IGNORE")]
    #[default]
    Ignore,
    #[sea_orm(string_value = "RETRY_ONCE")]
    RetryOnce,
}

#[derive(Debug, poem_openapi::Enum, EnumIter, Clone, Copy, DeriveActiveEnum, PartialEq, Eq, Serialize, Deserialize, Default)]
#[oai(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "String(Some(255))")]
pub enum ReachTemplateKind {
    #[sea_orm(string_value = "VCODE")]
    #[default]
    Vcode,
    #[sea_orm(string_value = "NOTICE")]
    Notice,
    #[sea_orm(string_value = "PROMOTE")]
    Promote,
}
