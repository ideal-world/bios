#![warn(clippy::unwrap_used)]
#![warn(clippy::dbg_macro)]

mod api;
pub mod conf_config;
pub mod conf_constants;
pub mod conf_initializer;
pub(crate) mod client;
pub(crate) use crate::conf_initializer::get_tardis_inst;
pub(crate) use crate::conf_initializer::get_tardis_inst_ref;
pub mod dto;
mod serv;
mod utils;
