use serde::{Deserialize, Serialize};
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};

use tardis::db::sea_orm;

use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::RbumScopeLevelKind;

/// Add request for resource item
///
/// 资源项添加请求
#[derive(Serialize, Deserialize, Debug)]
#[derive(poem_openapi::Object)]
pub struct RbumItemAddReq {
    /// Resource item id
    ///
    /// 资源项id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub id: Option<TrimString>,
    /// Resource item code
    ///
    /// 资源项编码
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub code: Option<TrimString>,
    /// Resource item name
    ///
    /// 资源项名称
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    /// Associated [resource kind](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) id
    ///
    /// 关联的[资源类型](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_kind_id: String,
    /// Associated [resource domain](crate::rbum::dto::rbum_domain_dto::RbumDomainDetailResp) id
    ///
    /// 关联的[资源域](crate::rbum::dto::rbum_domain_dto::RbumDomainDetailResp) id
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub rel_rbum_domain_id: String,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

impl Default for RbumItemAddReq {
    fn default() -> Self {
        Self {
            id: Default::default(),
            code: Default::default(),
            name: TrimString::from(""),
            rel_rbum_kind_id: Default::default(),
            rel_rbum_domain_id: Default::default(),
            scope_level: Default::default(),
            disabled: Default::default(),
        }
    }
}
/// Add request for resource item kernel
///
/// 资源项内核添加请求
///
/// Different from [`crate::rbum::dto::rbum_item_dto::RbumItemAddReq`], this object is used when there is a resource item extension table,
/// and the resource item contains kernel information (the ``rbum_item`` table) and extension information (the corresponding extension table).
///
/// 与 [`crate::rbum::dto::rbum_item_dto::RbumItemAddReq`] 不同，此对象用于有资源项扩展表的情况下使用，此时资源项包含了内核信息（``rbum_item``表）和扩展信息（对应的扩展表）。
#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RbumItemKernelAddReq {
    /// Resource item id
    ///
    /// 资源项id
    pub id: Option<TrimString>,
    /// Resource item code
    ///
    /// 资源项编码
    pub code: Option<TrimString>,
    /// Resource item name
    ///
    /// 资源项名称
    pub name: TrimString,
    /// Associated [resource kind](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) id
    ///
    /// 关联的[资源类型](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) id
    ///
    /// Special kind can be set, otherwise the default kind will be used.
    /// Note that setting special kind must ensure that the permissions are correct.
    ///
    /// 可以设置特殊的类型，否则将使用默认类型。
    /// 注意设置特殊类型必须确保权限正确。
    pub rel_rbum_kind_id: Option<String>,
    /// Associated [resource domain](crate::rbum::dto::rbum_domain_dto::RbumDomainDetailResp) id
    ///
    /// 关联的[资源域](crate::rbum::dto::rbum_domain_dto::RbumDomainDetailResp) id
    ///
    /// Special domain can be set, otherwise the default domain will be used.
    /// Note that setting special domain must ensure that the permissions are correct.
    ///
    /// 可以设置特殊的域，否则将使用默认域。
    /// 注意设置特殊域必须确保权限正确。
    pub rel_rbum_domain_id: Option<String>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

impl Default for RbumItemKernelAddReq {
    fn default() -> Self {
        Self {
            id: None,
            code: None,
            name: TrimString("".to_string()),
            rel_rbum_kind_id: None,
            rel_rbum_domain_id: None,
            scope_level: None,
            disabled: None,
        }
    }
}

/// Modify request for resource item kernel
///
/// 资源项内核修改请求
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct RbumItemKernelModifyReq {
    /// Resource item code
    ///
    /// 资源项编码
    pub code: Option<TrimString>,
    /// Resource item name
    ///
    /// 资源项名称
    pub name: Option<TrimString>,
    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

/// Resource item summary information
///
/// 资源项概要信息
#[derive(Serialize, Deserialize, Debug)]
#[derive(poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct RbumItemSummaryResp {
    /// Resource item id
    ///
    /// 资源项id
    pub id: String,
    /// Resource item code
    ///
    /// 资源项编码
    pub code: String,
    /// Resource item name
    ///
    /// 资源项名称
    pub name: String,
    /// Associated [resource kind](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) id
    ///
    /// 关联的[资源类型](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) id
    pub rel_rbum_kind_id: String,
    /// Associated [resource domain](crate::rbum::dto::rbum_domain_dto::RbumDomainDetailResp) id
    ///
    /// 关联的[资源域](crate::rbum::dto::rbum_domain_dto::RbumDomainDetailResp) id
    pub rel_rbum_domain_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}

/// Resource item detail information
///
/// 资源项详细信息
#[derive(Serialize, Deserialize, Debug)]
#[derive(poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct RbumItemDetailResp {
    /// Resource item id
    ///
    /// 资源项id
    pub id: String,
    /// Resource item code
    ///
    /// 资源项编码
    pub code: String,
    /// Resource item name
    ///
    /// 资源项名称
    pub name: String,
    /// Associated [resource kind](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) id
    ///
    /// 关联的[资源类型](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) id
    pub rel_rbum_kind_id: String,
    /// Associated [resource kind](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) name
    ///
    /// 关联的[资源类型](crate::rbum::dto::rbum_kind_dto::RbumKindDetailResp) 名称
    pub rel_rbum_kind_name: String,
    /// Associated [resource domain](crate::rbum::dto::rbum_domain_dto::RbumDomainDetailResp) id
    ///
    /// 关联的[资源域](crate::rbum::dto::rbum_domain_dto::RbumDomainDetailResp) id
    pub rel_rbum_domain_id: String,
    /// Associated [resource domain](crate::rbum::dto::rbum_domain_dto::RbumDomainDetailResp) name
    ///
    /// 关联的[资源域](crate::rbum::dto::rbum_domain_dto::RbumDomainDetailResp) 名称
    pub rel_rbum_domain_name: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}
