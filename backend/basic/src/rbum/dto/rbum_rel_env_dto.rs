use serde::{Deserialize, Serialize};
use tardis::chrono::{DateTime, Utc};
#[cfg(feature = "default")]
use tardis::db::sea_orm;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

use crate::rbum::rbum_enumeration::RbumRelEnvKind;

/// Add request for resource relationship environment condition
///
/// 资源关联环境条件添加请求
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumRelEnvAddReq {
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
    /// Associated [relationship](crate::rbum::dto::rbum_rel_dto::RbumRelDetailResp) id
    ///
    /// 关联的[资源关联](crate::rbum::dto::rbum_rel_dto::RbumRelDetailResp) id
    #[cfg_attr(feature = "default", oai(validator(min_length = "2", max_length = "255")))]
    pub rel_rbum_rel_id: String,
}

/// Modify request for resource relationship environment condition
///
/// 资源关联环境条件修改请求
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct RbumRelEnvModifyReq {
    /// Relationship environment value1
    ///
    /// 关联环境值1
    #[cfg_attr(feature = "default", oai(validator(min_length = "1", max_length = "2000")))]
    pub value1: Option<String>,
    /// Relationship environment value2
    ///
    /// 关联环境值2
    #[cfg_attr(feature = "default", oai(validator(min_length = "1", max_length = "2000")))]
    pub value2: Option<String>,
}

/// Resource relationship environment condition detail information
///
/// 资源关联环境条件详细信息
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct RbumRelEnvDetailResp {
    /// Relationship environment id
    ///
    /// 关联环境id
    pub id: String,
    /// Relationship environment type
    ///
    /// 关联的环境类型
    pub kind: RbumRelEnvKind,
    /// Relationship environment value1
    ///
    /// 关联环境值1
    pub value1: String,
    /// Relationship environment value2
    ///
    /// 关联环境值2
    pub value2: String,
    /// Associated [relationship](crate::rbum::dto::rbum_rel_dto::RbumRelDetailResp) id
    ///
    /// 关联的[资源关联](crate::rbum::dto::rbum_rel_dto::RbumRelDetailResp) id
    pub rel_rbum_rel_id: String,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
