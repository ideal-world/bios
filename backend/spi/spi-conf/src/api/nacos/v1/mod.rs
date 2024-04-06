mod auth;
mod config_service;
mod namespace;
use std::str::FromStr;

pub use auth::ConfNacosV1AuthApi;
pub use config_service::ConfNacosV1CsApi;
pub use namespace::ConfNacosV1NamespaceApi;
use poem::http::StatusCode;
use tardis::{basic::error::TardisError, web::poem};
pub type ConfNacosV1Api = (ConfNacosV1AuthApi, ConfNacosV1CsApi, ConfNacosV1NamespaceApi);

fn tardis_err_to_poem_err(e: TardisError) -> poem::Error {
    let status: StatusCode = StatusCode::from_str(&e.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    poem::Error::from_string(e.message, status)
}

fn missing_param(name: &str) -> poem::Error {
    poem::Error::from_string(format!("missing param {name}"), StatusCode::BAD_REQUEST)
}
