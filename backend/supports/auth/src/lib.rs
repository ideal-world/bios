#![warn(clippy::unwrap_used)]
#[cfg(feature = "web-server")]
pub(crate) mod api;
pub mod auth_config;
pub mod auth_constants;
pub mod auth_initializer;
pub mod dto;
mod error;
pub mod helper;
pub mod serv;
