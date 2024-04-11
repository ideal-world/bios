//! Basic enumerations
use serde::{Deserialize, Serialize};
use tardis::derive_more::Display;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

/// API classification
/// 
/// API分类
#[derive(Display, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Tags))]
pub enum ApiTag {
    /// Common Console, mostly starting with ``cc``, generally do not require authentication.
    /// 
    /// 公共类型, 多使用 ``cc`` 开头，一般不需要认证
    #[oai(rename = "Common Console")]
    Common,
    /// Tenant Console, mostly starting with ``ct``
    /// 
    /// 租户类型, 多使用 ``ct`` 开头
    #[oai(rename = "Tenant Console")]
    Tenant,
    /// App Console, mostly starting with ``ca``
    /// 
    /// 应用类型, 多使用 ``ca`` 开头
    #[oai(rename = "App Console")]
    App,
    /// System Console, mostly starting with ``cs``
    /// 
    /// 系统类型, 多使用 ``cs`` 开头
    #[oai(rename = "System Console")]
    System,
    /// Passport Console, mostly starting with ``cp``
    /// 
    /// 通行证类型, 多使用 ``cp`` 开头
    #[oai(rename = "Passport Console")]
    Passport,
    /// Interface Console, mostly starting with ``ci``, used for system-to-system calls, this type of interface generally uses ak/sk authentication
    /// 
    /// 接口类型, 多使用 ``ci`` 开头, 用于系统间调用, 此类型接口一般使用ak/sk认证
    #[oai(rename = "Interface Console")]
    Interface,
}

/// Basic query operator
/// 
/// 基础查询操作符
#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(poem_openapi::Enum))]
pub enum BasicQueryOpKind {
    #[oai(rename = "=")]
    Eq,
    #[oai(rename = "!=")]
    Ne,
    #[oai(rename = ">")]
    Gt,
    #[oai(rename = ">=")]
    Ge,
    #[oai(rename = "<")]
    Lt,
    #[oai(rename = "<=")]
    Le,
    #[oai(rename = "like")]
    Like,
    #[oai(rename = "not_like")]
    NotLike,
    #[oai(rename = "in")]
    In,
    #[oai(rename = "not_in")]
    NotIn,
    #[oai(rename = "is_null")]
    IsNull,
    #[oai(rename = "is_not_null")]
    IsNotNull,
    #[oai(rename = "is_null_or_empty")]
    IsNullOrEmpty,
}

impl BasicQueryOpKind {
    pub fn to_sql(&self) -> String {
        match self {
            BasicQueryOpKind::Eq => "=".to_string(),
            BasicQueryOpKind::Ne => "!=".to_string(),
            BasicQueryOpKind::Gt => ">".to_string(),
            BasicQueryOpKind::Ge => ">=".to_string(),
            BasicQueryOpKind::Lt => "<".to_string(),
            BasicQueryOpKind::Le => "<=".to_string(),
            BasicQueryOpKind::Like => "LIKE".to_string(),
            BasicQueryOpKind::NotLike => "NOT LIKE".to_string(),
            BasicQueryOpKind::In => "IN".to_string(),
            BasicQueryOpKind::NotIn => "NOT IN".to_string(),
            BasicQueryOpKind::IsNull => "IS NULL".to_string(),
            BasicQueryOpKind::IsNotNull => "IS NOT NULL".to_string(),
            BasicQueryOpKind::IsNullOrEmpty => "IS NULL".to_string(),
        }
    }
}
