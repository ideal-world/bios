use serde::{Deserialize, Serialize};
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm;
use tardis::web::poem_openapi;

use crate::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher, RbumItemRelFilterReq};
use crate::spi::spi_funs;

/// Add request for backend service
/// 
/// 添加后端服务的请求
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct SpiBsAddReq {
    /// Service name
    /// 
    /// 服务名称
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
    /// SPI service type Id. Used to partition the type corresponding to this service
    /// 
    /// SPI服务类型Id。用于分区该服务对应的类型
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub kind_id: TrimString,
    /// Connection URI
    /// 
    /// 连接URI
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub conn_uri: String,
    /// 连接用户名/凭证名
    /// 
    /// Connection username/credential name
    pub ak: TrimString,
    /// 连接密码/凭证密码
    /// 
    /// Connection password/credential password
    pub sk: TrimString,
    /// Extended information. Such as connection pool information
    /// 
    /// 扩展信息。比如连接池信息
    pub ext: String,
    /// Is private. Private service can only be used by one subject of request (tenant or application)
    /// 
    /// 是否私有。私有的服务只能用于一个请求主体（租户或应用）
    pub private: bool,
    /// Is disabled
    /// 
    /// 是否禁用
    pub disabled: Option<bool>,
}

/// Modify request for backend service
/// 
/// 修改后端服务的请求
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct SpiBsModifyReq {
    /// Service name
    /// 
    /// 服务名称
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: Option<TrimString>,
    /// SPI service type Id. Used to partition the type corresponding to this service
    /// 
    /// SPI服务类型Id。用于分区该服务对应的类型
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub kind_id: Option<TrimString>,
    /// Connection URI
    /// 
    /// 连接URI
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub conn_uri: Option<String>,
    /// Connection username/credential name
    /// 
    /// 连接用户名/凭证名
    #[oai(validator(min_length = "2"))]
    pub ak: Option<TrimString>,
    /// Connection password/credential password
    /// 
    /// 连接密码/凭证密码
    #[oai(validator(min_length = "2"))]
    pub sk: Option<TrimString>,
    /// Extended information. Such as connection pool information
    /// 
    /// 扩展信息。比如连接池信息
    pub ext: Option<String>,
    /// Is private. Private service can only be used by one subject of request (tenant or application)
    /// 
    /// 是否私有。私有的服务只能用于一个请求主体（租户或应用）
    pub private: Option<bool>,
    /// Is disabled
    /// 
    /// 是否禁用
    pub disabled: Option<bool>,
}

/// Backend service summary information
/// 
/// 后端服务的概要信息
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct SpiBsSummaryResp {
    /// Service Id
    /// 
    /// 服务Id
    pub id: String,
    /// Service name
    /// 
    /// 服务名称
    pub name: String,
    /// SPI service type Id
    /// 
    /// SPI服务类型Id
    pub kind_id: String,
    /// SPI service type code
    /// 
    /// SPI服务类型编码
    pub kind_code: String,
    /// SPI service type name
    /// 
    /// SPI服务类型名称
    pub kind_name: String,
    /// Connection URI
    /// 
    /// 连接URI
    pub conn_uri: String,
    /// Connection username/credential name
    /// 
    /// 连接用户名/凭证名
    pub ak: String,
    /// Connection password/credential password
    /// 
    /// 连接密码/凭证密码
    pub sk: String,
    /// Extended information. Such as connection pool information
    /// 
    /// 扩展信息。比如连接池信息
    pub ext: String,
    /// Is private. Private service can only be used by one subject of request (tenant or application)
    /// 
    /// 是否私有。私有的服务只能用于一个请求主体（租户或应用）
    pub private: bool,
    /// Is disabled
    /// 
    /// 是否禁用
    pub disabled: bool,
    /// Create time
    /// 
    /// 创建时间
    pub create_time: DateTime<Utc>,
    /// Update time
    /// 
    /// 更新时间
    pub update_time: DateTime<Utc>,
}

/// Backend service detail information
/// 
/// 后端服务的详细信息
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object, sea_orm::FromQueryResult))]
pub struct SpiBsDetailResp {
    /// Service Id
    /// 
    /// 服务Id
    pub id: String,
    /// Service name
    /// 
    /// 服务名称
    pub name: String,
    /// SPI service type Id
    /// 
    /// SPI服务类型Id
    pub kind_id: String,
    /// SPI service type code
    /// 
    /// SPI服务类型编码
    pub kind_code: String,
    /// SPI service type name
    /// 
    /// SPI服务类型名称
    pub kind_name: String,
    /// Connection URI
    /// 
    /// 连接URI
    pub conn_uri: String,
    /// Connection username/credential name
    /// 
    /// 连接用户名/凭证名
    pub ak: String,
    /// Connection password/credential password
    /// 
    /// 连接密码/凭证密码
    pub sk: String,
    /// Extended information. Such as connection pool information
    /// 
    /// 扩展信息。比如连接池信息
    pub ext: String,
    /// Is private. Private service can only be used by one subject of request (tenant or application)
    /// 
    /// 是否私有。私有的服务只能用于一个请求主体（租户或应用）
    pub private: bool,
    /// Is disabled
    /// 
    /// 是否禁用
    pub disabled: bool,
    /// Create time
    /// 
    /// 创建时间
    pub create_time: DateTime<Utc>,
    /// Update time
    /// 
    /// 更新时间
    pub update_time: DateTime<Utc>,
    /// Bound tenant or application Id
    /// 
    /// 绑定的租户或应用Id
    pub rel_app_tenant_ids: Vec<String>,
}

/// Backend service certificate information
/// 
/// 后端服务的凭证信息
#[derive(Serialize, Deserialize, Debug)]
pub struct SpiBsCertResp {
    /// SPI service type code
    /// 
    /// SPI服务类型编码
    pub kind_code: String,
    /// Connection URI
    /// 
    /// 连接URI
    pub conn_uri: String,
    /// Connection username/credential name
    /// 
    /// 连接用户名/凭证名
    pub ak: String,
    /// Connection password/credential password
    /// 
    /// 连接密码/凭证密码
    pub sk: String,
    /// Extended information. Such as connection pool information
    /// 
    /// 扩展信息。比如连接池信息
    pub ext: String,
    /// Is private. Private service can only be used by one subject of request (tenant or application)
    /// 
    /// 是否私有。私有的服务只能用于一个请求主体（租户或应用）
    pub private: bool,
}

impl SpiBsCertResp {
    pub fn bs_not_implemented(&self) -> TardisError {
        spi_funs::bs_not_implemented(&self.kind_code)
    }
}

/// Backend service query filter request
/// 
/// 后端服务的查询过滤请求
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
#[serde(default)]
pub struct SpiBsFilterReq {
    /// Basic filter
    /// 
    /// 基础过滤
    pub basic: RbumBasicFilterReq,
    /// Relational filter
    /// 
    /// 关联过滤
    pub rel: Option<RbumItemRelFilterReq>,
    /// Relational filter 2
    /// 
    /// 关联过滤2
    pub rel2: Option<RbumItemRelFilterReq>,
    /// Is private
    /// 
    /// 是否私有
    pub private: Option<bool>,
    /// SPI service type code
    /// 
    /// SPI服务类型编码
    pub kind_code: Option<String>,
    /// SPI service type codes
    /// 
    /// SPI服务类型编码集合
    pub kind_codes: Option<Vec<String>>,
    /// SPI service type Id
    /// 
    /// SPI服务类型Id
    pub kind_id: Option<String>,
    /// SPI service domain Id
    /// 
    /// SPI服务Domain Id
    pub domain_code: Option<String>,
}

impl RbumItemFilterFetcher for SpiBsFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
    fn rel(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel
    }
    fn rel2(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel2
    }
}
