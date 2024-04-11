//! Basic library for BIOS
//! BIOS的基础库
//!
//! This library provides the following functions:
//! 1. RBUM (Resource-Based Unified Model) model and implementation of common operations.
//! 1. SPI (Service Provider Interface) model and implementation of common operations.
//! 1. Common enumeration types.
//! 1. Common utility functions.
//! 1. Basic test support.
//!
//! 此类库提供了下功能：
//! 1. RBUM（基于资源的统一模型）模型及公共操作的实现
//! 1. SPI（服务提供接口）模型及公共操作的实现
//! 1. 通用枚举类型
//! 1. 通用工具函数
//! 1. 基础的测试支持
extern crate lazy_static;

pub mod dto;
pub mod enumeration;
pub mod helper;
pub mod process;
pub mod rbum;
pub mod spi;
#[cfg(feature = "test")]
pub mod test;

pub use enumeration::ApiTag;
use tardis::{TardisFuns, TardisFunsInst};

/// Extractor for ``TardisFunInst``
/// 提取 ``TardisFunsInst``
pub trait TardisFunInstExtractor {
    fn tardis_fun_inst(&self) -> TardisFunsInst;
}

/// Extract ``TardisFunInst`` from request path
/// 从请求路径中自动提取 ``TardisFunInst``
///
/// Get the first path segment from the request path as the service domain.
/// If there is a configuration parameter ``csm.X`` with the same name as the service domain, use this configuration parameter,
/// otherwise use the default configuration parameter.
/// 从请求路径中找到第一个路径段作为服务域名，如果存在与该服务域名同名的配置参数 ``csm.X``, 则使用该配置参数，否则使用默认配置参数.
#[cfg(feature = "default")]
impl TardisFunInstExtractor for tardis::web::poem::Request {
    fn tardis_fun_inst(&self) -> TardisFunsInst {
        let serv_domain = self.original_uri().path().split('/').collect::<Vec<&str>>()[1];
        TardisFuns::inst_with_db_conn(serv_domain.to_string(), None)
    }
}
