mod conf_auth;
mod conf_config_service_api;
mod conf_namespace_api;

pub use conf_auth::*;
pub use conf_config_service_api::*;
pub use conf_namespace_api::*;
pub type ConfCiApi = (ConfCiConfigServiceApi, ConfCiNamespaceApi, ConfCiAuthApi);
