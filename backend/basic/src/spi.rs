//! SPI (Service Provider Interface) module
//! 
//! SPI (服务提供接口) 模块
//!
//! The SPI in BIOS is used to provide different scenario-oriented capabilities.
//! These capabilities are abstracted into standardized service interfaces for other modules to call.
//! Based on these standardized service interfaces, different backend implementations can be extended and interfaced.
//! 
//! BIOS中的SPI用于提供面向不同场景的能力。这些能力抽象成标准化的服务接口，供其他模块调用。基于这些标准化的服务接口，可扩展对接不同的后端实现。
//!
//! # Example of invoke flow: full-text search service / 调用流程举例：全文搜索服务:
//!    
//!                                                                 +------------------+   
//!                                 +-----------+ +----------+    +-+backend-postgresql|   
//!                                 | spi-basic | |spi-search+----+ +------------------+--+
//!                                 +-----+-----+ +----+-----+    +-+backend-elasticsearch|
//!                        1. Init domain |            |            +---------------------+
//!                                       |            | 2. Init special entity            
//!                                       |            |                                   
//!                                       |            | 3. Init special API               
//!                    4. Init common API |            |                                   
//!                                       |            |                                   
//!                      +----------+     |            |                                   
//!                 ---->|Common API+---> |            |                                   
//!                      +----------+     |            |                                   
//!                5. Add backend service |            |                                   
//!                  (POST /ci/manage/bs) |            |                                   
//!                                       |            |     +-----------+                 
//!             6. Add cert to tenant/app |            | <---+Special API|<----            
//!         (PUT /:id/rel/:app_tenant_id) |            |     +-----------+                 
//!                                       |            | 7. Request some apis              
//!                                       |            |                                   
//!     8. Init and return backend client |            |                                   
//!                            (spi_funs) |            | 
//!                                       |            | 9. Call client to execute request
//!                                       |            |                                   
//!                                       |            | 10. Response data                 
//!                                       |            |   
//!    
//! # Key design:
//! 1. Reuse RBUM's ability
//!     1. Each SPI service has a ``rbum_domain`` for initializing domain objects.
//!        For example, the ``rbum_domain=spi-search`` of spi-search
//!     1. Each SPI service has one or more backend implementations,
//!        each corresponding to a ``rbum_kind``. Different SPI services can share the same ``rbum_kind``.
//!        For example, the ``rbum_kind=spi-bs-pg`` and ``rbum_kind=spi-bs-es`` of spi-search
//!     1. Each SPI backend implementation can have multiple connections, corresponding to ``rbum_item and extended spi_bs``.
//!        For example, multiple connections can be specified for ``spi-bs-pg`` of spi-search
//!     1. The connection information of each SPI backend implementation is stored in ``rbum_cert``
//!     1. The binding relationship between each SPI backend implementation and the corresponding tenant or application must be bound before use,
//!        and the binding relationship is stored in ``rbum_rel``, with the tag as ``spi_ident``
//! 1. No request authentication is done.
//!    The SPI service trusts the authentication information carried by the request (``owner`` in ``TardisContext``, corresponding to the Id of the tenant or application).
//!    The authentication logic will be implemented uniformly by the gateway
//! 1. Delayed initialization.
//!    The backend implementation of each SPI service is initialized (client generated) only when called for the first time to reduce resource consumption at startup.
//!    See [`crate::spi::spi_funs::SpiBsInst`] for details
//!
//! # 关键设计：
//! 1. 复用RBUM的能力
//!     1. 每个SPI服务都有一个``rbum_domain``，用于初始化领域对象。 如：spi-search的 ``rbum_domain=spi-search``
//!     1. 每个SPI服务有一个或多个后端实现，每个后端实现对应一个``rbum_kind``。不同的SPI服务可以共用相同的``rbum_kind``。如：spi-search的 ``rbum_kind=spi-bs-pg`` 和 ``rbum_kind=spi-bs-es``
//!     1. 每个SPI的后端实现可以有多个，对应于``rbum_item及扩展的spi_bs``。如可以为spi-search的``spi-bs-pg``指定多个连接
//!     1. 每个SPI后端实现的连接信息存储于``rbum_cert``
//!     1. 每个SPI后端实现的要绑定给对应的租户或应用后才能使用，绑定关系存储于``rbum_rel``，tag为``spi_ident``
//! 1. 不做请求认证。SPI服务信任请求带来的认证信息（``TardisContext``中的``owner``，对应于租户或应用的Id）。认证的逻辑将由网关统一实现
//! 1. 延时初始化。SPI服务的每个后端实现只有在第一次调用时才会初始化（生成client），以减少启动时的资源消耗。详见 [`crate::spi::spi_funs::SpiBsInst`]                                
#[cfg(feature = "default")]
pub mod api;
#[cfg(feature = "default")]
mod domain;
pub mod dto;
pub mod macros;
#[cfg(feature = "default")]
pub mod serv;
pub mod spi_constants;
pub mod spi_funs;
pub mod spi_initializer;
