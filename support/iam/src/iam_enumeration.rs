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

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum IamCertKernelKind {
    UserPwd,
    MailVCode,
    PhoneVCode,
    AkSk,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum IamCertExtKind {
    Ldap,
    OAuth2,
    /// No configuration exists,can't login in ,\
    /// supplier can be "gitlab/cmbd-pwd/cmbd-ssh"
    ThirdParty,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
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

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
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

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum IamRelKind {
    IamAccountRole,
    IamResRole,
    IamAccountApp,
    IamResApi,
    IamAccountRel,
    IamCertRel,
    IamOrgRel,

    IamProductSpec,
    IamCertProduct,
    IamCertSpec,
}

#[derive(Display, Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum IamResKind {
    #[default]
    Menu,
    Api,
    Ele,
    Product,
    Spec,
}

impl IamResKind {
    pub fn from_int(s: i16) -> TardisResult<IamResKind> {
        match s {
            0 => Ok(IamResKind::Menu),
            1 => Ok(IamResKind::Api),
            2 => Ok(IamResKind::Ele),
            3 => Ok(IamResKind::Product),
            4 => Ok(IamResKind::Spec),
            _ => Err(TardisError::format_error(&format!("invalid IamResKind: {s}"), "406-rbum-*-enum-init-error")),
        }
    }

    pub fn to_int(&self) -> i16 {
        match self {
            IamResKind::Menu => 0,
            IamResKind::Api => 1,
            IamResKind::Ele => 2,
            IamResKind::Product => 3,
            IamResKind::Spec => 4,
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

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
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

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
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

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString, Default)]
pub enum WayToAdd {
    ///同步账号同步凭证
    #[default]
    SynchronizeCert,
    ///同步账号不凭证
    NoSynchronizeCert,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString, Default)]
pub enum WayToDelete {
    ///什么也不做
    #[default]
    DoNotDelete,
    ///取消授权
    DeleteCert,
    ///禁用账号
    Disable,
    ///同步删除账号凭证
    DeleteAccount,
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString, Default)]
pub enum IamAccountLockStateKind {
    // 未锁定
    #[default]
    Unlocked,
    // 密码锁定
    PasswordLocked,
    // 长期未登录锁定
    LongTimeNoLoginLocked,
    // 人工锁定
    ManualLocked,
}

impl IamAccountLockStateKind {
    pub fn from_int(s: i16) -> TardisResult<IamAccountLockStateKind> {
        match s {
            0 => Ok(IamAccountLockStateKind::Unlocked),
            1 => Ok(IamAccountLockStateKind::PasswordLocked),
            2 => Ok(IamAccountLockStateKind::LongTimeNoLoginLocked),
            3 => Ok(IamAccountLockStateKind::ManualLocked),
            _ => Err(TardisError::format_error(&format!("invalid IamResKind: {s}"), "406-rbum-*-enum-init-error")),
        }
    }

    pub fn to_int(&self) -> i16 {
        match self {
            IamAccountLockStateKind::Unlocked => 0,
            IamAccountLockStateKind::PasswordLocked => 1,
            IamAccountLockStateKind::ManualLocked => 2,
            IamAccountLockStateKind::LongTimeNoLoginLocked => 3,
        }
    }
}

impl TryGetable for IamAccountLockStateKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = i16::try_get(res, pre, col)?;
        IamAccountLockStateKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implement")
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum IamAccountStatusKind {
    // 激活
    Active,
    // 休眠
    Dormant,
    // 注销
    Logout,
}
impl IamAccountStatusKind {
    pub fn parse(kind: &str) -> TardisResult<IamAccountStatusKind> {
        IamAccountStatusKind::from_str(kind).map_err(|_| TardisError::format_error(&format!("not account status type kind: {kind}"), "404-iam-account-status-not-exist"))
    }

    pub fn from_int(s: i16) -> TardisResult<IamAccountStatusKind> {
        match s {
            0 => Ok(IamAccountStatusKind::Active),
            1 => Ok(IamAccountStatusKind::Dormant),
            2 => Ok(IamAccountStatusKind::Logout),
            _ => Err(TardisError::format_error(&format!("invalid IamResKind: {s}"), "406-rbum-*-enum-init-error")),
        }
    }

    pub fn to_int(&self) -> i16 {
        match self {
            IamAccountStatusKind::Active => 0,
            IamAccountStatusKind::Dormant => 1,
            IamAccountStatusKind::Logout => 2,
        }
    }
}

impl TryGetable for IamAccountStatusKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = i16::try_get(res, pre, col)?;
        IamAccountStatusKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implement")
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum IamConfigDataTypeKind {
    // 月份
    Month,
    // 分钟
    Minute,
    // 小时
    Hour,
    // 天
    Day,
    // 数字
    Number,
    // 时间周期
    DatetimeRange,
    // 时间段
    TimeRange,
    // ip段
    Ips,
}
impl IamConfigDataTypeKind {
    pub fn parse(kind: &str) -> TardisResult<IamConfigDataTypeKind> {
        IamConfigDataTypeKind::from_str(kind).map_err(|_| TardisError::format_error(&format!("not config data type kind: {kind}"), "404-iam-config-data-type-not-exist"))
    }

    pub fn to_int(&self, value: String) -> Option<i64> {
        match self {
            IamConfigDataTypeKind::Number => Some(value.parse().unwrap_or_default()),
            IamConfigDataTypeKind::Month => None,
            IamConfigDataTypeKind::Minute => None,
            IamConfigDataTypeKind::Hour => None,
            IamConfigDataTypeKind::Day => None,
            IamConfigDataTypeKind::DatetimeRange => None,
            IamConfigDataTypeKind::TimeRange => None,
            IamConfigDataTypeKind::Ips => None,
        }
    }

    pub fn to_float(&self, value: String) -> Option<f64> {
        match self {
            IamConfigDataTypeKind::Number => Some(value.parse().unwrap_or_default()),
            IamConfigDataTypeKind::Month => None,
            IamConfigDataTypeKind::Minute => None,
            IamConfigDataTypeKind::Hour => None,
            IamConfigDataTypeKind::Day => None,
            IamConfigDataTypeKind::DatetimeRange => None,
            IamConfigDataTypeKind::TimeRange => None,
            IamConfigDataTypeKind::Ips => None,
        }
    }

    pub fn to_sec(&self, value: String) -> Option<i64> {
        match self {
            IamConfigDataTypeKind::Number => Some(value.parse().unwrap_or_default()),
            IamConfigDataTypeKind::Month => Some(value.parse::<i64>().unwrap_or_default() * 30 * 24 * 60 * 60),
            IamConfigDataTypeKind::Minute => Some(value.parse::<i64>().unwrap_or_default() * 60),
            IamConfigDataTypeKind::Hour => Some(value.parse::<i64>().unwrap_or_default() * 60 * 60),
            IamConfigDataTypeKind::Day => Some(value.parse::<i64>().unwrap_or_default() * 24 * 60 * 60),
            IamConfigDataTypeKind::DatetimeRange => None,
            IamConfigDataTypeKind::TimeRange => None,
            IamConfigDataTypeKind::Ips => None,
        }
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, strum::EnumString)]
pub enum IamConfigKind {
    /// 登陆时间限制
    AccessTime,
    /// 登陆Ip限制
    AuthIp,
    /// Token不活动失效 #分钟
    TokenExpire,
    /// 账号不活动锁定 #月
    AccountInactivityLock,
    /// 临时账号使用期限 #月
    AccountTemporaryExpire,
    /// 临时账号休眠期限 #月
    AccountTemporarySleepExpire,
    /// 临时账号休眠注销 #月
    AccountTemporarySleepRemoveExpire,
    /// 审计日志容量 #GB
    AuditLogCapacity,
    /// 在线人数限制 #人
    MaxOnline,
}

impl IamConfigKind {
    pub fn parse(kind: &str) -> TardisResult<IamConfigKind> {
        IamConfigKind::from_str(kind).map_err(|_| TardisError::format_error(&format!("not config kind: {kind}"), "404-iam-config-kind-not-exist"))
    }
}
