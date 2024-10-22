use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::Display;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
#[cfg(feature = "default")]
use tardis::db::sea_orm;
#[cfg(feature = "default")]
use tardis::db::sea_orm::{DbErr, QueryResult, TryGetError, TryGetable};
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

/// Scope level kind
///
/// 作用域层级类型
#[derive(Display, Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "default", derive(poem_openapi::Enum))]
pub enum RbumScopeLevelKind {
    /// Private
    ///
    /// 私有
    ///
    /// Only the current level is visible.
    ///
    /// 仅当前层级可见。
    Private,
    /// （全局）完全公开
    ///
    /// （Global）Fully open
    Root,
    /// The first level
    ///
    /// 第一层
    ///
    /// The current level and its descendants are visible.
    ///
    /// 当前层及其子孙层可见。
    L1,
    /// The second level
    ///
    /// 第二层
    ///
    /// The current level and its descendants are visible.
    L2,
    /// The third level
    ///
    /// 第三层
    ///
    /// The current level and its descendants are visible.
    L3,
}

impl<'de> Deserialize<'de> for RbumScopeLevelKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|s| match s.to_lowercase().as_str() {
            "private" => Ok(RbumScopeLevelKind::Private),
            "root" => Ok(RbumScopeLevelKind::Root),
            "l1" => Ok(RbumScopeLevelKind::L1),
            "l2" => Ok(RbumScopeLevelKind::L2),
            "l3" => Ok(RbumScopeLevelKind::L3),
            _ => Err(serde::de::Error::custom(format!("invalid RbumScopeLevelKind: {s}"))),
        })
    }
}

impl RbumScopeLevelKind {
    pub fn from_int(s: i16) -> TardisResult<RbumScopeLevelKind> {
        match s {
            -1 => Ok(RbumScopeLevelKind::Private),
            0 => Ok(RbumScopeLevelKind::Root),
            1 => Ok(RbumScopeLevelKind::L1),
            2 => Ok(RbumScopeLevelKind::L2),
            3 => Ok(RbumScopeLevelKind::L3),
            _ => Err(TardisError::format_error(&format!("invalid RbumScopeLevelKind: {s}"), "406-rbum-*-enum-init-error")),
        }
    }

    pub fn to_int(&self) -> i16 {
        match self {
            RbumScopeLevelKind::Private => -1,
            RbumScopeLevelKind::Root => 0,
            RbumScopeLevelKind::L1 => 1,
            RbumScopeLevelKind::L2 => 2,
            RbumScopeLevelKind::L3 => 3,
        }
    }
}

#[cfg(feature = "default")]
impl TryGetable for RbumScopeLevelKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = i16::try_get(res, pre, col)?;
        RbumScopeLevelKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        i16::try_get_by(res, index).map(RbumScopeLevelKind::from_int)?.map_err(|e| TryGetError::DbErr(DbErr::Custom(format!("invalid scope level: {e}"))))
    }
}

/// Certificate relationship kind
///
///
/// 凭证关联的类型
#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(poem_openapi::Enum))]
pub enum RbumCertRelKind {
    /// Resource item
    ///
    /// 资源项
    Item,
    /// Resource set
    ///
    /// 资源集
    Set,
    /// Resource relation
    ///
    /// 资源关联
    Rel,
}

impl RbumCertRelKind {
    pub fn from_int(s: i16) -> TardisResult<RbumCertRelKind> {
        match s {
            0 => Ok(RbumCertRelKind::Item),
            1 => Ok(RbumCertRelKind::Set),
            2 => Ok(RbumCertRelKind::Rel),
            _ => Err(TardisError::format_error(&format!("invalid RbumCertRelKind: {s}"), "406-rbum-*-enum-init-error")),
        }
    }

    pub fn to_int(&self) -> i16 {
        match self {
            RbumCertRelKind::Item => 0,
            RbumCertRelKind::Set => 1,
            RbumCertRelKind::Rel => 2,
        }
    }
}

#[cfg(feature = "default")]
impl TryGetable for RbumCertRelKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = i16::try_get(res, pre, col)?;
        RbumCertRelKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}

/// Resource certificate configuration status kind
///
/// 资源凭证配置状态类型
#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(poem_openapi::Enum))]
pub enum RbumCertConfStatusKind {
    /// Disabled
    ///
    /// 禁用
    Disabled,
    /// Enabled
    ///
    /// 启用
    Enabled,
}

impl RbumCertConfStatusKind {
    pub fn from_int(s: i16) -> TardisResult<RbumCertConfStatusKind> {
        match s {
            0 => Ok(RbumCertConfStatusKind::Disabled),
            1 => Ok(RbumCertConfStatusKind::Enabled),
            _ => Err(TardisError::format_error(&format!("invalid RbumCertConfStatusKind: {s}"), "406-rbum-*-enum-init-error")),
        }
    }

    pub fn to_int(&self) -> i16 {
        match self {
            RbumCertConfStatusKind::Disabled => 0,
            RbumCertConfStatusKind::Enabled => 1,
        }
    }
}

/// Resource certificate status kind
///
/// 资源凭证状态类型
#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(poem_openapi::Enum))]
pub enum RbumCertStatusKind {
    /// Disabled
    ///
    /// 禁用
    Disabled,
    /// Enabled
    ///
    /// 启用
    Enabled,
    /// Pending
    ///
    /// 正在处理
    Pending,
}

impl RbumCertStatusKind {
    pub fn from_int(s: i16) -> TardisResult<RbumCertStatusKind> {
        match s {
            0 => Ok(RbumCertStatusKind::Disabled),
            1 => Ok(RbumCertStatusKind::Enabled),
            2 => Ok(RbumCertStatusKind::Pending),
            _ => Err(TardisError::format_error(&format!("invalid RbumCertStatusKind: {s}"), "406-rbum-*-enum-init-error")),
        }
    }

    pub fn to_int(&self) -> i16 {
        match self {
            RbumCertStatusKind::Disabled => 0,
            RbumCertStatusKind::Enabled => 1,
            RbumCertStatusKind::Pending => 2,
        }
    }
}

#[cfg(feature = "default")]
impl TryGetable for RbumCertStatusKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = i16::try_get(res, pre, col)?;
        RbumCertStatusKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}

/// Resource relation kind
///
/// 资源关联的类型
#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(poem_openapi::Enum))]
pub enum RbumRelFromKind {
    /// Resource item
    ///
    /// 资源项
    Item,
    /// Resource set
    ///
    /// 资源集
    Set,
    /// Resource set category(node)
    ///
    /// 资源集分类（节点）
    SetCate,
    /// Resource certificate
    ///
    /// 资源凭证
    Cert,
}

impl RbumRelFromKind {
    pub fn from_int(s: i16) -> TardisResult<RbumRelFromKind> {
        match s {
            0 => Ok(RbumRelFromKind::Item),
            1 => Ok(RbumRelFromKind::Set),
            2 => Ok(RbumRelFromKind::SetCate),
            3 => Ok(RbumRelFromKind::Cert),
            _ => Err(TardisError::format_error(&format!("invalid RbumRelFromKind: {s}"), "406-rbum-*-enum-init-error")),
        }
    }

    pub fn to_int(&self) -> i16 {
        match self {
            RbumRelFromKind::Item => 0,
            RbumRelFromKind::Set => 1,
            RbumRelFromKind::SetCate => 2,
            RbumRelFromKind::Cert => 3,
        }
    }
}

#[cfg(feature = "default")]
impl TryGetable for RbumRelFromKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = i16::try_get(res, pre, col)?;
        RbumRelFromKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}

/// Resource relation environment kind
///
/// 资源关联环境类型
///
/// Used to associate resources with restrictions.
///
/// 用于给资源关联加上限制条件。
#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(poem_openapi::Enum))]
pub enum RbumRelEnvKind {
    /// Datetime range
    ///
    /// 日期时间范围
    ///
    /// Format: ``UNIX timestamp``
    DatetimeRange,
    /// Time range
    ///
    /// 时间范围
    ///
    /// Format: ``hhmmss``.
    ///
    /// hh   = two digits of hour (00 through 23) (am/pm NOT allowed)
    /// mm   = two digits of minute (00 through 59)
    /// ss   = two digits of second (00 through 59)
    TimeRange,
    /// IP list
    ///
    /// IP地址
    ///
    /// Format: ``ip1,ip2,ip3``
    Ips,
    /// Call frequency
    ///
    /// 调用频率
    ///
    /// Request value must be less than or equal to the set value.
    ///
    /// 请求的值要小于等于设置的值。
    CallFrequency,
    /// Call count
    ///
    /// 调用次数
    ///
    /// Request value must be less than or equal to the set value.
    ///
    /// 请求的值要小于等于设置的值。
    CallCount,
}

impl RbumRelEnvKind {
    pub fn from_int(s: i16) -> TardisResult<RbumRelEnvKind> {
        match s {
            0 => Ok(RbumRelEnvKind::DatetimeRange),
            1 => Ok(RbumRelEnvKind::TimeRange),
            2 => Ok(RbumRelEnvKind::Ips),
            3 => Ok(RbumRelEnvKind::CallFrequency),
            4 => Ok(RbumRelEnvKind::CallCount),
            _ => Err(TardisError::format_error(&format!("invalid RbumRelEnvKind: {s}"), "406-rbum-*-enum-init-error")),
        }
    }

    pub fn to_int(&self) -> i16 {
        match self {
            RbumRelEnvKind::DatetimeRange => 0,
            RbumRelEnvKind::TimeRange => 1,
            RbumRelEnvKind::Ips => 2,
            RbumRelEnvKind::CallFrequency => 3,
            RbumRelEnvKind::CallCount => 4,
        }
    }
}

#[cfg(feature = "default")]
impl TryGetable for RbumRelEnvKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = i16::try_get(res, pre, col)?;
        RbumRelEnvKind::from_int(s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}

/// Resource set category(node) query kind
///
/// 资源集分类（节点）的查询类型
#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(poem_openapi::Enum))]
pub enum RbumSetCateLevelQueryKind {
    /// Current layer and descendant layer
    ///
    /// 当前层及子孙层
    CurrentAndSub,
    /// Current layer and grandfather layer
    ///
    /// 当前层及祖父层
    CurrentAndParent,
    /// Descendant layer
    ///
    /// 子孙层
    Sub,
    /// Grandfather layer
    ///
    /// 祖父层
    Parent,
    /// Current layer only
    ///
    /// 仅当前层
    Current,
}

/// Data kind
///
/// 数据类型
#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(poem_openapi::Enum, strum::EnumString))]
pub enum RbumDataTypeKind {
    String,
    Number,
    Boolean,
    Date,
    DateTime,
    Json,
    Strings,
    Numbers,
    Booleans,
    Dates,
    DateTimes,
    Array,
    Label,
}

#[cfg(feature = "default")]
impl TryGetable for RbumDataTypeKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        RbumDataTypeKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}

/// Widget kind
///
/// （前端）控件类型
#[derive(Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "default", derive(poem_openapi::Enum, strum::EnumString))]
pub enum RbumWidgetTypeKind {
    Input,
    InputTxt,
    InputNum,
    Textarea,
    Number,
    Date,
    DateTime,
    Upload,
    Radio,
    Button,
    Checkbox,
    Switch,
    Select,
    MultiSelect,
    Link,
    CodeEditor,
    /// Display group subtitles, ``datatype = String & value is empty``
    ///
    /// 显示组标题，``datatype = String & 值为空``
    Container,
    /// Json fields : ``datatype = Json && all parent_attr_name = current attribute``
    ///
    /// Json字段，``datatype = Json && 所有 parent_attr_name = 当前属性``
    Control,
    /// Sub fields :  ``datatype = Array && all parent_attr_name = current attribute``, The value of the json array is stored to the current field.
    ///
    /// 子字段，``datatype = Array && 所有 parent_attr_name = 当前属性``，将json数组的值存储到当前字段。
    Group,
}

#[cfg(feature = "default")]
impl TryGetable for RbumWidgetTypeKind {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let s = String::try_get(res, pre, col)?;
        RbumWidgetTypeKind::from_str(&s).map_err(|_| TryGetError::DbErr(DbErr::RecordNotFound(format!("{pre}:{col}"))))
    }

    fn try_get_by<I: sea_orm::ColIdx>(_res: &QueryResult, _index: I) -> Result<Self, TryGetError> {
        panic!("not implemented")
    }
}
