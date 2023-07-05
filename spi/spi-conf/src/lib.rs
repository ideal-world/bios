#![warn(clippy::unwrap_used)]
#![warn(clippy::dbg_macro)]

mod api;
pub mod conf_config;
pub mod conf_constants;
pub mod conf_initializer;
pub(crate) use crate::conf_initializer::get_tardis_inst;
pub mod dto;
mod serv;
mod utils;
