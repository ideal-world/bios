use serde::{Deserialize, Serialize};
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

use crate::rbum::dto::rbum_rel_attr_dto::RbumRelAttrDetailResp;
use crate::rbum::dto::rbum_rel_dto::{RbumRelAddReq, RbumRelDetailResp};
use crate::rbum::dto::rbum_rel_env_dto::RbumRelEnvDetailResp;
use crate::rbum::rbum_enumeration::RbumRelEnvKind;

/// Add request for resource relationship aggregation
///
/// 资源关联聚合添加请求
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumRelAggAddReq {
    /// Relationship information
    ///
    /// 关联信息
    pub rel: RbumRelAddReq,
    /// Relationship attribute information
    ///
    /// 关联属性信息
    pub attrs: Vec<RbumRelAttrAggAddReq>,
    /// Relationship environment information
    ///
    /// 关联环境信息
    pub envs: Vec<RbumRelEnvAggAddReq>,
}

/// Add request for resource relationship attribute aggregation
///
/// 资源关联属性聚合添加请求
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumRelAttrAggAddReq {
    /// Condition qualifier
    ///
    /// 条件限定符
    ///
    /// if true, it means the limitation of the relationship source,
    /// otherwise it is the limitation of the relationship target resource.
    ///
    /// 如果为true，表示关联来源方的限定，否则为关联目标方资源的限定。
    pub is_from: bool,
    /// Relationship attribute value
    ///
    /// 关联属性值
    #[cfg_attr(feature = "default", oai(validator(min_length = "0", max_length = "2000")))]
    pub value: String,
    /// Relationship attribute name
    ///
    /// 关联属性名称
    ///
    /// Redundant field.
    ///
    /// 冗余字段。
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub name: String,
    /// Whether to only record
    ///
    /// 是否仅记录
    ///
    /// If true, this condition is only used for records and does not participate in the judgment of whether the relationship is established.
    ///
    /// 如果为true，该条件仅用于记录，不参与判断关联关系是否建立。
    pub record_only: bool,
    /// Associated [resource kind attribute](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) id
    ///
    /// 关联的[资源类型属性](crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp) id
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_kind_attr_id: String,
}

/// Add request for resource relationship environment aggregation
///
/// 资源关联环境聚合添加请求
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumRelEnvAggAddReq {
    /// Relationship environment type
    ///
    /// 关联的环境类型
    pub kind: RbumRelEnvKind,
    /// Relationship environment value1
    ///
    /// 关联环境值1
    #[cfg_attr(feature = "default", oai(validator(min_length = "1", max_length = "2000")))]
    pub value1: String,
    /// Relationship environment value2
    ///
    /// 关联环境值2
    #[cfg_attr(feature = "default", oai(validator(min_length = "1", max_length = "2000")))]
    pub value2: Option<String>,
}

/// Resource relationship aggregation detail information
///
/// 资源关联聚合详细信息
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumRelAggResp {
    /// Relationship information
    ///
    /// 关联信息
    pub rel: RbumRelDetailResp,
    /// Relationship attribute information
    ///
    /// 关联属性信息
    pub attrs: Vec<RbumRelAttrDetailResp>,
    /// Relationship environment information
    ///
    /// 关联环境信息
    pub envs: Vec<RbumRelEnvDetailResp>,
}
