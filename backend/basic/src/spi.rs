
//! SPI (Service Provider Interface) module.
//! 
//! The SPI in BIOS is used to provide different scenario-oriented capabilities.
//! These capabilities are abstracted into standardized service interfaces for other modules to call. 
//! Based on these standardized service interfaces, different backend implementations can be extended and interfaced.
//! BIOS中的SPI用于提供面向不同场景的能力。这些能力抽象成标准化的服务接口，供其他模块调用。基于这些标准化的服务接口，可扩展对接不同的后端实现。
//! 
//! Example of invoke flow: full-text search service:
//! 调用流程举例：全文搜索服务:
//! 
//!                                                             +------------------+   
//!                             +-----------+ +----------+    +-+backend-postgresql|   
//!                             | spi-basic | |spi-search+----+ +------------------+--+
//!                             +-----+-----+ +----+-----+    +-+backend-elasticsearch|
//!                    1. Init domain |            |            +---------------------+
//!                                   |            | 2. Init special entity            
//!                                   |            |                                   
//!                                   |            | 3. Init special API               
//!                4. Init common API |            |                                   
//!                                   |            |                                   
//!                  +----------+     |            |                                   
//!             ---->|Common API+---> |            |                                   
//!                  +----------+     |            |                                   
//!            5. Add backend service |            |                                   
//!              (POST /ci/manage/bs) |            |                                   
//!                                   |            |     +-----------+                 
//!         6. Add cert to tenant/app |            | <---+Special API|<----            
//!     (PUT /:id/rel/:app_tenant_id) |            |     +-----------+                 
//!                                   |            | 7. Request some apis              
//!                                   |            |                                   
//! 8. Init and return backend client |            |                                   
//!                        (spi_funs) |            | 9. Call client to execute request 
//!                                   |            |                                   
//!                                   |            | 10. Response data                 
//!                                   |            |                                                                
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
