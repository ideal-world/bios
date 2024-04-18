use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
#[cfg(feature = "default")]
use tardis::db::sea_orm;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

/// Add request for resource domain
///
/// 资源域添加请求
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumDomainAddReq {
    /// Resource domain code
    ///
    /// 资源域编码
    ///
    /// Global unique
    ///
    /// 全局唯一
    ///
    /// Which is required to conform to the host specification in the uri, matching the regular: ^[a-z0-9-.]+$.
    ///
    /// 需要符合uri中的host规范，匹配正则：^[a-z0-9-.]+$。
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub code: TrimString,
    /// Resource domain name
    ///
    /// 资源域名称
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub name: TrimString,
    /// Resource domain note
    ///
    /// 资源域备注
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub note: Option<String>,
    /// Resource domain icon
    ///
    /// 资源域图标
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub icon: Option<String>,
    /// Resource domain sort
    ///
    /// 资源域排序
    pub sort: Option<i64>,

    pub scope_level: Option<RbumScopeLevelKind>,
}

/// Modify request for resource domain
///
/// 资源域修改请求
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumDomainModifyReq {
    /// Resource domain name
    ///
    /// 资源域名称
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub name: Option<TrimString>,
    /// Resource domain note
    ///
    /// 资源域备注
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "2000")))]
    pub note: Option<String>,
    /// Resource domain icon
    ///
    /// 资源域图标
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "1000")))]
    pub icon: Option<String>,
    /// Resource domain sort
    ///
    /// 资源域排序
    pub sort: Option<i64>,
    /// Resource domain scope level
    ///
    /// 资源域作用域级别
    ///
    /// Default is ``private``
    ///
    /// 默认为``私有``
    pub scope_level: Option<RbumScopeLevelKind>,
}

/// Resource domain summary information
///
/// 资源域概要信息
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumDomainSummaryResp {
    /// Resource domain id
    ///
    /// 资源域id
    pub id: String,
    /// Resource domain code
    ///
    /// 资源域编码
    pub code: String,
    /// Resource domain name
    ///
    /// 资源域名称
    pub name: String,
    /// Resource domain icon
    ///
    /// 资源域图标
    pub icon: String,
    /// Resource domain sort
    ///
    /// 资源域排序
    pub sort: i64,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}

/// Resource domain detail information
///
/// 资源域详细信息
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumDomainDetailResp {
    /// Resource domain id
    ///
    /// 资源域id
    pub id: String,
    /// Resource domain code
    ///
    /// 资源域编码
    pub code: String,
    /// Resource domain name
    ///
    /// 资源域名称
    pub name: String,
    /// Resource domain note
    ///
    /// 资源域备注
    pub note: String,
    /// Resource domain icon
    ///
    /// 资源域图标
    pub icon: String,
    pub sort: i64,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
}
