use crate::rbum::constants::RBUM_ITEM_TENANT_CODE_LEN;

pub(crate) mod constants;
#[cfg(feature = "default")]
pub(crate) mod domain;
pub mod dto;
pub mod enumeration;
#[cfg(feature = "default")]
pub mod initializer;
#[cfg(feature = "default")]
pub mod serv;

pub fn get_tenant_code_from_app_code(app_code: &str) -> String {
    app_code[..RBUM_ITEM_TENANT_CODE_LEN].to_string()
}