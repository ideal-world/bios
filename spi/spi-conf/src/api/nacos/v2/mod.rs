// mod auth;
mod config_service;
mod namespace;
use std::str::FromStr;

pub use config_service::ConfNacosV2CsApi;
pub use namespace::ConfNacosV2NamespaceApi;
use poem::http::StatusCode;
use tardis::{basic::error::TardisError, web::poem};
pub type ConfNacosV2Api = (ConfNacosV2CsApi, ConfNacosV2NamespaceApi);

fn tardis_err_to_poem_err(e: TardisError) -> poem::Error {
    let status: StatusCode = StatusCode::from_str(&e.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    poem::Error::from_string(e.message, status)
}
